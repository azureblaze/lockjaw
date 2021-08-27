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
use crate::directx::{check_result, ComPtr};
use cubit::ApplicationComponent;
use lockjaw::injectable;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::ptr::null_mut;
use winapi::shared::dxgi::{IDXGIAdapter1, DXGI_ADAPTER_DESC1, DXGI_ADAPTER_FLAG_SOFTWARE};
use winapi::shared::dxgi1_3::CreateDXGIFactory2;
use winapi::shared::dxgi1_4::IDXGIFactory4;
use winapi::shared::dxgi1_6::IDXGIAdapter4;
use winapi::shared::minwindef::TRUE;
use winapi::shared::winerror::{DXGI_ERROR_NOT_FOUND, SUCCEEDED};
use winapi::um::d3d12::{D3D12CreateDevice, ID3D12Device};
use winapi::um::d3d12sdklayers::{
    ID3D12InfoQueue, D3D12_MESSAGE_SEVERITY_CORRUPTION, D3D12_MESSAGE_SEVERITY_ERROR,
    D3D12_MESSAGE_SEVERITY_WARNING,
};
use winapi::um::d3dcommon::D3D_FEATURE_LEVEL_11_0;
use winapi::um::unknwnbase::IUnknown;
use winapi::Interface;

lockjaw::prologue!("src/directx/dx_device.rs");

pub struct DxDevice {
    device: ComPtr<ID3D12Device>,
}

#[injectable(scope: ApplicationComponent, container: RefCell)]
impl DxDevice {
    #[inject]
    pub fn new() -> DxDevice {
        unsafe {
            let mut device: ComPtr<ID3D12Device> = ComPtr::null();

            check_result(D3D12CreateDevice(
                get_dxgi_adapter().unwrap().as_unknown(),
                D3D_FEATURE_LEVEL_11_0,
                &ID3D12Device::uuidof(),
                device.get_cvoid_address(),
            ))
            .unwrap();

            let mut info_queue: ComPtr<ID3D12InfoQueue> = ComPtr::null();
            if SUCCEEDED(device.transmute(&mut info_queue, &ID3D12InfoQueue::uuidof())) || true {
                info_queue.SetBreakOnSeverity(D3D12_MESSAGE_SEVERITY_CORRUPTION, TRUE);
                info_queue.SetBreakOnSeverity(D3D12_MESSAGE_SEVERITY_ERROR, TRUE);
                info_queue.SetBreakOnSeverity(D3D12_MESSAGE_SEVERITY_WARNING, TRUE);
                info_queue.transmute(&mut device, &ID3D12Device::uuidof());
            }

            DxDevice { device }
        }
    }
}

unsafe fn get_dxgi_adapter() -> Option<ComPtr<IDXGIAdapter4>> {
    let mut dxgi_factory: ComPtr<IDXGIFactory4> = ComPtr::null();
    check_result(CreateDXGIFactory2(
        0,
        &IDXGIFactory4::uuidof(),
        dxgi_factory.get_cvoid_address(),
    ))
    .unwrap();

    let mut i = 0u32;
    let mut max_dedicated_video_memory = 0usize;
    let mut dxgi_adapter1: ComPtr<IDXGIAdapter1> = ComPtr::null();
    let mut dxgi_adapter4: ComPtr<IDXGIAdapter4> = ComPtr::null();
    while dxgi_factory.EnumAdapters1(i, dxgi_adapter1.get_address()) != DXGI_ERROR_NOT_FOUND {
        i += 1;

        let mut dxgi_adapter_desc1 = std::mem::zeroed::<DXGI_ADAPTER_DESC1>();
        dxgi_adapter1.GetDesc1(&mut dxgi_adapter_desc1);
        if dxgi_adapter_desc1.Flags & DXGI_ADAPTER_FLAG_SOFTWARE != 0 {
            continue;
        }
        if dxgi_adapter_desc1.DedicatedVideoMemory <= max_dedicated_video_memory {
            continue;
        }
        if !SUCCEEDED(D3D12CreateDevice(
            dxgi_adapter1.get() as *mut IUnknown,
            D3D_FEATURE_LEVEL_11_0,
            &ID3D12Device::uuidof(),
            null_mut(),
        )) {
            continue;
        }

        max_dedicated_video_memory = dxgi_adapter_desc1.DedicatedVideoMemory;
        dxgi_adapter1.transmute(&mut dxgi_adapter4, &IDXGIAdapter4::uuidof());
    }
    if dxgi_adapter4.get().is_null() {
        None
    } else {
        Some(dxgi_adapter4)
    }
}

impl Deref for DxDevice {
    type Target = ID3D12Device;

    fn deref(&self) -> &Self::Target {
        self.device.deref()
    }
}

impl DerefMut for DxDevice {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.device.deref_mut()
    }
}
