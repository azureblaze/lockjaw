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
use winapi::um::d3d12::{ID3D12DescriptorHeap, D3D12_DESCRIPTOR_HEAP_DESC, D3D12_DESCRIPTOR_HEAP_TYPE, D3D12_CPU_DESCRIPTOR_HANDLE};
use winapi::Interface;

lockjaw::prologue!("src/directx/descriptor_heap.rs");

pub struct DescriptorHeap {
    descriptor_heap: ComPtr<ID3D12DescriptorHeap>,
}

#[injectable]
impl DescriptorHeap {
    #[factory(visibility: "pub")]
    pub fn new<'a>(
        device: &'a RefCell<DxDevice>,
        #[runtime] type_: D3D12_DESCRIPTOR_HEAP_TYPE,
        #[runtime] num_descriptors: u32,
    ) -> Self {
        unsafe {
            let mut descriptor_heap = ComPtr::<ID3D12DescriptorHeap>::null();
            check_result(device.borrow_mut().CreateDescriptorHeap(
                &D3D12_DESCRIPTOR_HEAP_DESC {
                    Type: type_,
                    NumDescriptors: num_descriptors,
                    Flags: 0,
                    NodeMask: 0,
                },
                &ID3D12DescriptorHeap::uuidof(),
                descriptor_heap.get_cvoid_address(),
            ))
            .unwrap();
            Self { descriptor_heap }
        }
    }

    pub(in crate::directx) fn get_cpu_descriptor_handle_for_heap_start(&self) -> D3D12_CPU_DESCRIPTOR_HANDLE{
        unsafe {
            self.descriptor_heap.GetCPUDescriptorHandleForHeapStart()
        }
    }
}