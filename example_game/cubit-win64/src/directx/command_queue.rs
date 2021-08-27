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
use crate::directx::fence::Fence;
use crate::directx::{check_result, ComPtr};
use crate::graphics::renderer::RendererComponent;
use lockjaw::injectable;
use std::cell::RefCell;
use winapi::um::d3d12::{ID3D12CommandQueue, D3D12_COMMAND_LIST_TYPE_DIRECT, D3D12_COMMAND_QUEUE_DESC, D3D12_COMMAND_QUEUE_FLAG_NONE, D3D12_COMMAND_QUEUE_PRIORITY_NORMAL, ID3D12CommandList};
use winapi::um::unknwnbase::IUnknown;
use winapi::Interface;
use crate::directx::command_list::CommandList;

lockjaw::prologue!("src/directx/command_queue.rs");

pub struct DxCommandQueue<'c> {
    command_queue: ComPtr<ID3D12CommandQueue>,
    fence: &'c RefCell<Fence>,
}

#[injectable(scope: RendererComponent, container: RefCell)]
impl<'c> DxCommandQueue<'c> {
    #[inject]
    pub fn new(device: &'c RefCell<DxDevice>, fence: &'c RefCell<Fence>) -> Self {
        unsafe {
            let mut desc = std::mem::zeroed::<D3D12_COMMAND_QUEUE_DESC>();
            desc.Type = D3D12_COMMAND_LIST_TYPE_DIRECT;
            desc.Priority = D3D12_COMMAND_QUEUE_PRIORITY_NORMAL as i32;
            desc.Flags = D3D12_COMMAND_QUEUE_FLAG_NONE;
            desc.NodeMask = 0u32;

            let mut command_queue = ComPtr::<ID3D12CommandQueue>::null();
            check_result(device.borrow_mut().CreateCommandQueue(
                &desc,
                &ID3D12CommandQueue::uuidof(),
                command_queue.get_cvoid_address(),
            ))
                .unwrap();

            DxCommandQueue {
                command_queue,
                fence,
            }
        }
    }

    pub fn signal(&self) -> u64 {
        let fence_value = self.fence.borrow_mut().increment();
        unsafe {
            check_result(
                self.command_queue
                    .Signal(self.fence.borrow().ptr(), fence_value),
            )
                .unwrap();
        }
        fence_value
    }

    pub fn flush(&self) {
        let fence_value_for_signal = self.signal();
        self.fence
            .borrow()
            .wait_for_fence_value(fence_value_for_signal);
    }

    pub fn execute_command_lists(&self, command_lists: &Vec<&mut CommandList>) {
        unsafe {
            let pointers: Vec<*mut ID3D12CommandList> = command_lists.iter().map(|list| list.get_address() as *mut ID3D12CommandList).collect();
            self.command_queue.ExecuteCommandLists(command_lists.len() as u32, pointers.as_ptr())
        }
    }

    pub fn as_unknown(&mut self) -> &mut IUnknown {
        self.command_queue.as_unknown()
    }
}
