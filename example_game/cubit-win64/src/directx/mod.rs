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
use std::ffi::c_void;
use std::fmt::{Display, Formatter};
use std::mem::transmute;
use std::ops::{Deref, DerefMut};
use std::ptr::{null, null_mut};
use winapi::shared::guiddef::REFGUID;
use winapi::shared::winerror::{HRESULT, SUCCEEDED};
use winapi::um::d3d12::D3D12GetDebugInterface;
use winapi::um::d3d12sdklayers::ID3D12Debug;
use winapi::um::unknwnbase::IUnknown;
use winapi::um::winbase::{FormatMessageW, FORMAT_MESSAGE_FROM_SYSTEM};
use winapi::um::winnt::{LANG_NEUTRAL, MAKELANGID, SUBLANG_DEFAULT};
use winapi::Interface;

pub mod command_allocator;
pub mod command_list;
pub mod command_queue;
pub mod descriptor_heap;
pub mod dx_device;
pub mod fence;
pub mod swap_chain;

pub struct ComPtr<T> {
    ptr: *mut T,
}

impl<T> ComPtr<T> {
    pub fn null() -> ComPtr<T> {
        ComPtr { ptr: null_mut() }
    }

    pub fn as_unknown(&mut self) -> &mut IUnknown {
        unsafe { &mut *std::mem::transmute::<*mut T, *mut IUnknown>(self.ptr) }
    }

    pub fn get(&self) -> *mut T {
        self.ptr
    }

    pub fn get_address(&mut self) -> *mut *mut T {
        &mut self.ptr as *mut *mut T
    }

    pub fn get_cvoid_address(&mut self) -> *mut *mut c_void {
        unsafe { std::mem::transmute(&mut self.ptr) }
    }

    pub unsafe fn transmute<U>(&mut self, target: &mut ComPtr<U>, uuid: REFGUID) -> HRESULT {
        if !target.ptr.is_null() {
            target.as_unknown().Release();
            target.ptr = null_mut();
        }
        let result = self
            .as_unknown()
            .QueryInterface(uuid, target.get_cvoid_address()) as HRESULT;
        if SUCCEEDED(result) {
            self.ptr = null_mut();
        }
        result
    }

    pub fn reset(&mut self) {
        if self.ptr == null_mut() {
            return;
        }
        unsafe {
            self.as_unknown().Release();
        }
        self.ptr = null_mut();
    }
}

impl<T> Drop for ComPtr<T> {
    fn drop(&mut self) {
        self.reset()
    }
}

impl<T> Deref for ComPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl<T> DerefMut for ComPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}

#[derive(Debug, Clone)]
pub struct DxError {
    message: String,
}

impl Display for DxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub fn check_result(result: HRESULT) -> Result<(), DxError> {
    if result >= 0 {
        return Ok(());
    }
    unsafe {
        let mut buffer: [u16; 4096] = [0; 4096];
        FormatMessageW(
            FORMAT_MESSAGE_FROM_SYSTEM,
            null(),
            result as u32,
            MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT) as u32,
            transmute(&mut buffer),
            4096,
            null_mut(),
        );
        let end = buffer.iter().position(|&c| c == 0).unwrap_or(4096);
        let message = String::from_utf16_lossy(&buffer[0..end]);

        Err(DxError { message })
    }
}
pub fn enable_debug_layer() {
    unsafe {
        let mut debug_interface: ComPtr<ID3D12Debug> = ComPtr::null();
        check_result(D3D12GetDebugInterface(
            &ID3D12Debug::uuidof(),
            debug_interface.get_cvoid_address(),
        ))
        .unwrap();
        debug_interface.EnableDebugLayer();
    }
}
