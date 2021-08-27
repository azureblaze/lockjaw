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

pub mod window;

use crate::application::window::{WindowImpl, WindowImplFactory, WindowRegistry};
use crate::directx;
use cubit::core::{Application, ApplicationPrivate, Game, Window};
use cubit::ApplicationComponent;
use lockjaw::{component_visible, injectable, module, Cl, Provider};
use std::cell::{Cell, RefCell};
use std::ptr::null_mut;
use std::rc::Rc;
use std::time::Instant;
use winapi::shared::minwindef::HINSTANCE;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::{
    DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE, WM_QUIT,
};

lockjaw::prologue!("src/application/mod.rs");

#[component_visible]
pub(in crate::application) struct ApplicationImpl<'a> {
    hinstance: HINSTANCE,
    window_factory: WindowImplFactory<'a>,
    window_registry: &'a RefCell<WindowRegistry<'a>>,
    game: Provider<'a, Cl<'a, dyn Game>>,
    running: Cell<bool>,
}

#[injectable(scope : ApplicationComponent)]
impl<'a> ApplicationImpl<'a> {
    #[inject]
    pub fn new(
        window_factory: WindowImplFactory<'a>,
        window_registry: &'a RefCell<WindowRegistry<'a>>,
        game: Provider<'a, Cl<'a, dyn Game>>,
    ) -> Self {
        unsafe {
            Self {
                hinstance: GetModuleHandleW(std::ptr::null()),
                window_factory,
                window_registry,
                game,
                running: Cell::new(true),
            }
        }
    }

    pub fn get_hinstance(&self) -> HINSTANCE {
        self.hinstance
    }
}

impl<'a> Application<'a> for ApplicationImpl<'a> {
    fn quit(&self) {
        self.running.set(false);
    }

    fn create_window(&self) -> Rc<dyn Window + 'a> {
        let window: Rc<WindowImpl> = Rc::new(self.window_factory.new());
        self.window_registry.borrow_mut().add_window(window.clone());
        window
    }
}

impl<'a> ApplicationPrivate<'a> for ApplicationImpl<'a> {
    fn initialize(&self) {
        directx::enable_debug_layer();
    }

    fn start(&self) {
        unsafe {
            let game = self.game.get();
            game.initialize();

            let mut msg = std::mem::zeroed::<MSG>();
            let mut last_update = Instant::now();
            while self.running.get() {
                while PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) != 0 {
                    TranslateMessage(&mut msg);
                    DispatchMessageW(&mut msg);
                    if msg.message == WM_QUIT {
                        self.quit();
                        return;
                    }
                }
                let dt = Instant::now().duration_since(last_update);
                last_update = Instant::now();
                game.update(dt.as_secs_f32());
                game.render();
            }
        }
    }
}

#[component_visible]
struct AppModule;

#[module(install_in: ApplicationComponent)]
impl AppModule {
    #[binds]
    pub fn bind_application<'a>(_impl: &'a ApplicationImpl) -> Cl<dyn Application<'a>> {}
    #[binds]
    pub fn bind_application_private<'a>(
        _impl: &'a ApplicationImpl,
    ) -> Cl<dyn ApplicationPrivate<'a>> {
    }
}
