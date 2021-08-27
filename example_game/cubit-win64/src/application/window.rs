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

use crate::application::ApplicationImpl;
use crate::graphics::renderer::Renderer;
use crate::graphics::renderer::RendererFactory;
use crate::marshal::to_cwstr;
use cubit::core::Window;
use cubit::graphics::RenderTarget;
use cubit::ApplicationComponent;
use cubit::{get_application_component, StartupListener};
use lockjaw::{component_visible, entry_point, injectable, module, Cl};
use std::cell::RefCell;
use std::collections::HashMap;
use std::mem::zeroed;
use std::ptr::{null, null_mut};
use std::rc::Rc;
use winapi::shared::minwindef::{FALSE, HINSTANCE, LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HBRUSH, HWND, RECT};
use winapi::um::winuser::{
    AdjustWindowRect, CreateWindowExW, DefWindowProcW, GetClientRect, GetSystemMetrics,
    LoadCursorW, PostQuitMessage, RegisterClassExW, ShowWindow, COLOR_WINDOW, CS_HREDRAW,
    CS_VREDRAW, IDC_ARROW, SM_CXSCREEN, SM_CYSCREEN, SW_SHOW, WM_DESTROY, WM_EXITSIZEMOVE,
    WNDCLASSEXW, WS_OVERLAPPEDWINDOW,
};

lockjaw::prologue!("src/application/window.rs");

#[component_visible]
pub(in crate::application) struct WindowImpl<'a> {
    handle: HWND,
    renderer: Renderer<'a>,
}

#[injectable]
impl WindowImpl<'_> {
    #[factory(visibility: "pub(in crate::application)")]
    pub fn new<'a>(
        application: &'a ApplicationImpl<'a>,
        renderer_factory: RendererFactory<'a>,
    ) -> WindowImpl<'a> {
        unsafe {
            let mut window_size = RECT {
                left: 0,
                top: 0,
                right: 1920,
                bottom: 1080,
            };

            let x_pos = (GetSystemMetrics(SM_CXSCREEN) - window_size.right) / 2;
            let y_pos = (GetSystemMetrics(SM_CYSCREEN) - window_size.bottom) / 2;

            AdjustWindowRect(&mut window_size, WS_OVERLAPPEDWINDOW, FALSE);
            let handle = CreateWindowExW(
                0,
                to_cwstr(&WINDOW_CLASS_NAME).as_ptr(),
                null(),
                WS_OVERLAPPEDWINDOW,
                x_pos,
                y_pos,
                window_size.right - window_size.left,
                window_size.bottom - window_size.top,
                null_mut(),
                null_mut(),
                application.hinstance,
                null_mut(),
            );

            let renderer = renderer_factory.new(handle);
            renderer.initialize();
            WindowImpl { handle, renderer }
        }
    }

    pub unsafe fn on_window_proc(&self, umsg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match umsg {
            WM_DESTROY => {
                PostQuitMessage(0);
            }
            WM_EXITSIZEMOVE => {
                let mut client_rect = zeroed::<RECT>();
                GetClientRect(self.handle, &mut client_rect);
                self.renderer.resize(
                    client_rect.right - client_rect.left,
                    client_rect.bottom - client_rect.top,
                );
            }
            _ => {
                return DefWindowProcW(self.handle, umsg, wparam, lparam);
            }
        }
        return 0;
    }
}

impl<'a> Window for WindowImpl<'a> {
    fn show(&self) {
        unsafe {
            ShowWindow(self.handle, SW_SHOW);
        }
    }

    fn begin<'w>(&'w self) -> Box<dyn RenderTarget + 'w> {
        Box::new(self.renderer.prepare_frame())
    }
}

const WINDOW_CLASS_NAME: &str = "WindowClass1";

pub fn register_window_class(hinstance: HINSTANCE) {
    unsafe {
        let window_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: null_mut(),
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hbrBackground: COLOR_WINDOW as HBRUSH,
            lpszMenuName: null_mut(),
            lpszClassName: to_cwstr(&WINDOW_CLASS_NAME).as_ptr(),
            hIconSm: null_mut(),
        };
        RegisterClassExW(&window_class);
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    umsg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let entry_point: &dyn AppEntryPoint = <dyn AppEntryPoint>::get(get_application_component());

    if let Some(window) = entry_point.window_registry().borrow().get(&hwnd) {
        return window.on_window_proc(umsg, wparam, lparam);
    }

    DefWindowProcW(hwnd, umsg, wparam, lparam)
}

#[component_visible]
struct AppModule {}

#[module(install_in: ApplicationComponent)]
impl AppModule {
    #[provides]
    #[into_vec]
    pub fn provide_register_window_class_startup_listener<'a>(
        app: &'a ApplicationImpl,
    ) -> Cl<'a, dyn StartupListener> {
        Cl::Val(Box::new(move || register_window_class(app.get_hinstance())))
    }
}

pub struct WindowRegistry<'a> {
    windows: HashMap<HWND, Rc<WindowImpl<'a>>>,
}

#[injectable(scope: ApplicationComponent, container: RefCell)]
impl<'a> WindowRegistry<'a> {
    #[inject]
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
        }
    }

    pub fn add_window(&mut self, window: Rc<WindowImpl<'a>>) {
        let handle = window.handle;
        self.windows.insert(handle, window);
    }

    pub fn get(&self, hwnd: &HWND) -> Option<&Rc<WindowImpl<'a>>> {
        self.windows.get(hwnd)
    }
}

#[entry_point(install_in: ApplicationComponent)]
trait AppEntryPoint {
    fn window_registry(&self) -> &RefCell<WindowRegistry>;
}
