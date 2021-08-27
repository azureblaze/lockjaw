/*
Copyright 2021 Google LLC

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use crate::directx::command_allocator::CommandAllocator;
use crate::directx::command_list::CommandList;
use crate::directx::command_queue::DxCommandQueue;
use crate::directx::descriptor_heap::DescriptorHeap;
use crate::directx::descriptor_heap::DescriptorHeapFactory;
use crate::directx::dx_device::DxDevice;
use crate::directx::fence::Fence;
use crate::directx::{check_result, ComPtr};
use crate::graphics::renderer::RendererComponent;
use lockjaw::{injectable, Provider};
use std::cell::RefCell;
use std::cmp::max;
use std::mem::zeroed;
use std::ptr::{null, null_mut};
use winapi::shared::dxgi::{DXGI_SWAP_CHAIN_DESC, DXGI_SWAP_EFFECT_FLIP_DISCARD};
use winapi::shared::dxgi1_2::{
    IDXGISwapChain1, DXGI_ALPHA_MODE_UNSPECIFIED, DXGI_SCALING_STRETCH, DXGI_SWAP_CHAIN_DESC1,
};
use winapi::shared::dxgi1_3::CreateDXGIFactory2;
use winapi::shared::dxgi1_4::IDXGIFactory4;
use winapi::shared::dxgi1_5::IDXGISwapChain4;
use winapi::shared::dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM;
use winapi::shared::dxgitype::{DXGI_SAMPLE_DESC, DXGI_USAGE_RENDER_TARGET_OUTPUT};
use winapi::shared::minwindef::FALSE;
use winapi::shared::windef::{HWND, RECT};
use winapi::um::d3d12::{
    ID3D12Resource, D3D12_CPU_DESCRIPTOR_HANDLE, D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
};
use winapi::um::winuser::GetClientRect;
use winapi::Interface;
use cubit::graphics::Color;

const BACK_BUFFER_SIZE: u32 = 3;

lockjaw::prologue!("src/directx/swap_chain.rs");

pub struct Frame {
    command_allocator: CommandAllocator,
    back_buffer: ComPtr<ID3D12Resource>,
    fence_value: u64,
}

#[injectable]
impl Frame {
    #[inject]
    pub fn new(command_allocator: CommandAllocator) -> Frame {
        Frame {
            command_allocator,
            back_buffer: ComPtr::null(),
            fence_value: 0,
        }
    }
}

pub struct FrameRef<'r, 'c>
    where
        'c: 'r,
{
    swap_chain: &'r mut SwapChain<'c>,
    frame_index: usize,
}

impl<'r, 'c> FrameRef<'r, 'c> {
    pub fn get_command_allocator(&mut self) -> &mut CommandAllocator {
        &mut self.get_frame().command_allocator
    }

    pub fn get_back_buffer(&mut self) -> &mut ComPtr<ID3D12Resource> {
        &mut self.get_frame().back_buffer
    }

    pub fn clear_render_target_view(&mut self, clear_color: &Color) {
        self.swap_chain.clear_render_target_view(clear_color);
    }

    pub fn present(&mut self) {
        self.swap_chain.present(self.frame_index);
    }

    fn get_frame(&mut self) -> &mut Frame {
        &mut self.swap_chain.frames[self.frame_index]
    }

    pub fn get_index(&self) -> usize {
        self.frame_index
    }
}

pub struct SwapChain<'r> {
    swap_chain: ComPtr<IDXGISwapChain4>,
    frames: Vec<Frame>,
    frame_index: usize,
    rtv_descriptor_size: usize,
    descriptor_heap: DescriptorHeap,
    width: u32,
    height: u32,
    command_list: Provider<'r, &'r RefCell<CommandList>>,
    command_queue: &'r RefCell<DxCommandQueue<'r>>,
    fence: &'r RefCell<Fence>,
    device: &'r RefCell<DxDevice>,
}

#[injectable(scope: RendererComponent, container: RefCell)]
impl<'r> SwapChain<'r> {
    #[inject]
    pub fn new(
        hwnd: HWND,
        device: &'r RefCell<DxDevice>,
        command_queue: &'r RefCell<DxCommandQueue<'r>>,
        command_list: Provider<'r, &'r RefCell<CommandList>>,
        frame_provider: Provider<'r, Frame>,
        descriptor_heap_factory: DescriptorHeapFactory,
        fence: &'r RefCell<Fence>,
    ) -> Self {
        unsafe {
            let mut dxgi_factory: ComPtr<IDXGIFactory4> = ComPtr::null();
            check_result(CreateDXGIFactory2(
                0,
                &IDXGIFactory4::uuidof(),
                dxgi_factory.get_cvoid_address(),
            ))
                .unwrap();
            let (width, height) = get_client_size(hwnd);

            let swap_chain_desc = DXGI_SWAP_CHAIN_DESC1 {
                Width: width,
                Height: height,
                Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                Stereo: FALSE,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: BACK_BUFFER_SIZE,
                Scaling: DXGI_SCALING_STRETCH,
                SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                AlphaMode: DXGI_ALPHA_MODE_UNSPECIFIED,
                Flags: 0,
            };

            let mut swap_chain_1: ComPtr<IDXGISwapChain1> = ComPtr::null();
            check_result(dxgi_factory.CreateSwapChainForHwnd(
                command_queue.borrow_mut().as_unknown(),
                hwnd,
                &swap_chain_desc,
                null(),
                null_mut(),
                swap_chain_1.get_address(),
            ))
                .unwrap();

            let mut swap_chain: ComPtr<IDXGISwapChain4> = ComPtr::null();
            swap_chain_1.transmute(&mut swap_chain, &IDXGISwapChain1::uuidof());

            let mut frames = Vec::new();
            for _ in 0..BACK_BUFFER_SIZE {
                frames.push(frame_provider.get());
            }

            let rtv_descriptor_size = device
                .borrow_mut()
                .GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV)
                as usize;
            Self {
                frame_index: swap_chain.GetCurrentBackBufferIndex() as usize,
                swap_chain,
                frames,
                rtv_descriptor_size,
                descriptor_heap: descriptor_heap_factory
                    .new(D3D12_DESCRIPTOR_HEAP_TYPE_RTV, BACK_BUFFER_SIZE),
                width,
                height,
                command_list,
                command_queue,
                fence,
                device,
            }
        }
    }

    pub fn new_frame<'s>(&'s mut self) -> FrameRef<'s, 'r> {
        unsafe {
            self.frame_index = self.swap_chain.GetCurrentBackBufferIndex() as usize;
            let frame = &mut self.frames[self.frame_index];
            self.fence
                .borrow()
                .wait_for_fence_value(frame.fence_value);
            frame.command_allocator.reset();
            return FrameRef {
                frame_index: self.frame_index,
                swap_chain: self,
            };
        }
    }

    pub fn get_frame<'s>(&'s mut self, frame_index: usize) -> FrameRef<'s, 'r> {
        assert_eq!(frame_index, self.frame_index);
        return FrameRef {
            frame_index,
            swap_chain: self,
        };
    }

    pub fn clear_render_target_view(&self, clear_color: &Color) {
        let rtv = self.get_rtv();
        self.command_list
            .get()
            .borrow()
            .clear_render_target_view(rtv, clear_color);
    }

    fn present(&mut self, frame_index: usize) {
        unsafe {
            self.swap_chain.Present(0, 0);
            self.frames[frame_index].fence_value = self.command_queue.borrow_mut().signal();
        }
    }

    pub fn get_current_frame_index(&self) -> usize {
        self.frame_index
    }

    pub fn update_render_target_views(&mut self) {
        unsafe {
            let mut rtv_handle = self.descriptor_heap.get_cpu_descriptor_handle_for_heap_start();
            for i in 0..self.frames.len() {
                let mut back_buffer = ComPtr::<ID3D12Resource>::null();
                check_result(self.swap_chain.GetBuffer(
                    i as u32,
                    &ID3D12Resource::uuidof(),
                    back_buffer.get_cvoid_address(),
                ))
                    .unwrap();

                self.device.borrow_mut().CreateRenderTargetView(
                    back_buffer.get(),
                    std::ptr::null(),
                    rtv_handle.clone(),
                );
                self.frames[i].back_buffer = back_buffer;
                rtv_handle.ptr += self.rtv_descriptor_size;
            }
        }
    }

    pub fn get_rtv(&self) -> D3D12_CPU_DESCRIPTOR_HANDLE {
        let mut rtv = self.descriptor_heap.get_cpu_descriptor_handle_for_heap_start();
        rtv.ptr += self.rtv_descriptor_size as usize * self.frame_index;
        rtv
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == self.width && height == self.height {
            return;
        }

        println!("resize {}x{}", width, height);

        self.width = max(1, width);
        self.height = max(1, height);

        let current_fence_value = self.frames[self.frame_index].fence_value;
        for i in 0..self.frames.len() {
            self.frames[i].back_buffer.reset();
            self.frames[i].fence_value = current_fence_value;
        }
        unsafe {
            let mut swap_chain_desc = zeroed::<DXGI_SWAP_CHAIN_DESC>();
            check_result(self.swap_chain.GetDesc(&mut swap_chain_desc)).unwrap();
            check_result(self.swap_chain.ResizeBuffers(
                self.frames.len() as u32,
                width,
                height,
                swap_chain_desc.BufferDesc.Format,
                swap_chain_desc.Flags,
            ))
                .unwrap();

            self.frame_index = self.swap_chain.GetCurrentBackBufferIndex() as usize;

            self.update_render_target_views();
        }
    }

    pub fn default_command_allocator(&mut self) -> &mut CommandAllocator {
        &mut self.frames[self.frame_index].command_allocator
    }
}

fn get_client_size(hwnd: HWND) -> (u32, u32) {
    unsafe {
        let mut rect = zeroed::<RECT>();
        GetClientRect(hwnd, &mut rect);
        (
            max(1, rect.right - rect.left) as u32,
            max(1, rect.bottom - rect.top) as u32,
        )
    }
}
