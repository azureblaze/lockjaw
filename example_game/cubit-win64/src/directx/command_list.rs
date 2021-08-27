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

use crate::directx::dx_device::DxDevice;
use crate::directx::swap_chain::SwapChain;
use crate::directx::{check_result, ComPtr};
use crate::graphics::renderer::RendererComponent;
use lockjaw::injectable;
use std::cell::RefCell;
use std::ptr::{null_mut, null};
use winapi::um::d3d12::{ID3D12GraphicsCommandList, D3D12_COMMAND_LIST_TYPE_DIRECT, D3D12_CPU_DESCRIPTOR_HANDLE, D3D12_RESOURCE_BARRIER, D3D12_RESOURCE_STATE_PRESENT, D3D12_RESOURCE_STATE_RENDER_TARGET, ID3D12Resource, D3D12_RESOURCE_STATES, D3D12_RESOURCE_BARRIER_TYPE_TRANSITION, D3D12_RESOURCE_BARRIER_FLAG_NONE, D3D12_RESOURCE_TRANSITION_BARRIER, D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES};
use winapi::Interface;
use cubit::graphics::Color;
use crate::directx::command_allocator::CommandAllocator;
use std::mem::zeroed;

lockjaw::prologue!("src/directx/command_list.rs");

pub struct CommandList {
    command_list: ComPtr<ID3D12GraphicsCommandList>,
}

#[injectable(scope: RendererComponent, container: RefCell)]
impl CommandList {
    #[inject]
    pub fn new(device: &RefCell<DxDevice>, swap_chain: &RefCell<SwapChain>) -> Self {
        let mut command_list = ComPtr::<ID3D12GraphicsCommandList>::null();
        unsafe {
            check_result(
                device.borrow_mut().CreateCommandList(
                    0,
                    D3D12_COMMAND_LIST_TYPE_DIRECT,
                    swap_chain
                        .borrow_mut()
                        .default_command_allocator()
                        .get_ptr(),
                    null_mut(),
                    &ID3D12GraphicsCommandList::uuidof(),
                    command_list.get_cvoid_address(),
                ),
            )
                .unwrap();
            check_result(command_list.Close()).unwrap();

            CommandList { command_list }
        }
    }

    pub fn get_address(&self) -> *mut ID3D12GraphicsCommandList {
        self.command_list.get()
    }

    pub(in crate::directx) fn clear_render_target_view(&self, render_target_view: D3D12_CPU_DESCRIPTOR_HANDLE, color: &Color) {
        unsafe {
            let clear_color = [color.r, color.g, color.b, color.a];
            self.command_list.ClearRenderTargetView(render_target_view, &clear_color, 0, null())
        }
    }

    pub fn reset(&self, allocator: &CommandAllocator) -> i32 {
        unsafe {
            self.command_list.Reset(allocator.get_ptr(), null_mut())
        }
    }

    pub fn render_target_barrier(&self, back_buffer: &mut ComPtr<ID3D12Resource>) {
        let barrier = resource_barrier_transition(
            back_buffer.get(),
            D3D12_RESOURCE_STATE_PRESENT,
            D3D12_RESOURCE_STATE_RENDER_TARGET,
        );
        self.resource_barrier(1, &barrier);
    }

    pub fn present_barrier(&self, back_buffer: &mut ComPtr<ID3D12Resource>) {
        let barrier = resource_barrier_transition(
            back_buffer.get(),
            D3D12_RESOURCE_STATE_RENDER_TARGET, D3D12_RESOURCE_STATE_PRESENT,
        );
        self.resource_barrier(1, &barrier);
    }

    fn resource_barrier(&self, num_barriers: u32, p_barriers: *const D3D12_RESOURCE_BARRIER) {
        unsafe {
            self.command_list.ResourceBarrier(num_barriers, p_barriers)
        }
    }


    pub fn close(&self) -> i32 {
        unsafe {
            self.command_list.Close()
        }
    }
}

fn resource_barrier_transition(
    resource: *mut ID3D12Resource,
    state_before: D3D12_RESOURCE_STATES,
    state_after: D3D12_RESOURCE_STATES,
) -> D3D12_RESOURCE_BARRIER {
    unsafe {
        let mut result = zeroed::<D3D12_RESOURCE_BARRIER>();
        result.Type = D3D12_RESOURCE_BARRIER_TYPE_TRANSITION;
        result.Flags = D3D12_RESOURCE_BARRIER_FLAG_NONE;
        *result.u.Transition_mut() = D3D12_RESOURCE_TRANSITION_BARRIER {
            pResource: resource,
            Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
            StateBefore: state_before,
            StateAfter: state_after,
        };

        result
    }
}