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
use lockjaw::injectable;
use std::cell::RefCell;
use winapi::um::d3d12::{ID3D12CommandAllocator, D3D12_COMMAND_LIST_TYPE_DIRECT};
use winapi::Interface;

lockjaw::prologue!("src/directx/command_allocator.rs");

pub struct CommandAllocator {
    command_allocator: ComPtr<ID3D12CommandAllocator>,
}

#[injectable]
impl CommandAllocator {
    #[inject]
    pub fn new(device: &RefCell<DxDevice>) -> Self {
        unsafe {
            let mut command_allocator = ComPtr::<ID3D12CommandAllocator>::null();
            check_result(device.borrow_mut().CreateCommandAllocator(
                D3D12_COMMAND_LIST_TYPE_DIRECT,
                &ID3D12CommandAllocator::uuidof(),
                command_allocator.get_cvoid_address(),
            ))
            .unwrap();
            Self { command_allocator }
        }
    }

    pub fn get_ptr(&self) -> *mut ID3D12CommandAllocator {
        self.command_allocator.get()
    }

    pub fn reset(&self) -> i32{
        unsafe {
            self.command_allocator.Reset()
        }
    }
}