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
use crate::directx::{check_result, ComPtr};
use crate::graphics::renderer::RendererComponent;
use lockjaw::injectable;
use std::cell::RefCell;
use std::ptr::{null, null_mut};
use winapi::shared::minwindef::FALSE;
use winapi::um::d3d12::{ID3D12Fence, D3D12_FENCE_FLAG_NONE};
use winapi::um::synchapi::{CreateEventW, WaitForSingleObject};
use winapi::um::winnt::HANDLE;
use winapi::Interface;

lockjaw::prologue!("src/directx/fence.rs");

pub struct Fence {
    fence: ComPtr<ID3D12Fence>,
    fence_event: HANDLE,
    fence_value: u64,
}

#[injectable(scope: RendererComponent, container: RefCell)]
impl Fence {
    #[inject]
    pub fn new(device: &RefCell<DxDevice>) -> Self {
        let mut fence = ComPtr::<ID3D12Fence>::null();
        unsafe {
            check_result(device.borrow_mut().CreateFence(
                0,
                D3D12_FENCE_FLAG_NONE,
                &ID3D12Fence::uuidof(),
                fence.get_cvoid_address(),
            ))
            .unwrap();
        }
        Fence {
            fence,
            fence_event: Fence::create_event_handle(),
            fence_value: 0,
        }
    }

    pub fn ptr(&self) -> *mut ID3D12Fence {
        self.fence.get()
    }

    fn create_event_handle() -> HANDLE {
        unsafe {
            let fence_event = CreateEventW(null_mut(), FALSE, FALSE, null());
            assert!(!fence_event.is_null(), "Failed to create fence event.");
            fence_event
        }
    }

    pub fn increment(&mut self) -> u64 {
        self.fence_value += 1;
        self.fence_value
    }

    pub fn wait_for_fence_value(&self, fence_value: u64) {
        unsafe {
            if self.fence.GetCompletedValue() < fence_value {
                check_result(
                    self.fence
                        .SetEventOnCompletion(fence_value, self.fence_event),
                )
                .unwrap();
                WaitForSingleObject(self.fence_event, u32::MAX);
            }
        }
    }
}
