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

pub mod core;
pub mod graphics;
pub mod os;

use crate::core::ApplicationPrivate;
use lockjaw::{component_visible, define_component, entry_point, module, Cl};
use std::borrow::Borrow;
use std::error::Error;

lockjaw::prologue!("src/lib.rs");

#[define_component]
pub trait ApplicationComponent {}

pub trait StartupListener {
    fn on_startup(&self) -> ();
}

impl<F> StartupListener for F
where
    F: Fn(),
{
    fn on_startup(&self) {
        self()
    }
}

#[component_visible]
struct AppAlwaysModule {}

#[module(install_in: ApplicationComponent)]
impl AppAlwaysModule {
    #[multibinds]
    pub fn startup_listeners() -> Vec<Cl<'static, dyn StartupListener>> {}
}

#[entry_point(install_in: ApplicationComponent)]
trait AppEntryPoint {
    fn startup_listeners(&'_ self) -> Vec<Cl<'_, dyn StartupListener>>;

    fn app(&self) -> Cl<dyn ApplicationPrivate>;
}

static mut APPLICATION_COMPONENT: Option<&Box<dyn ApplicationComponent>> = None;

pub fn get_application_component() -> &'static dyn ApplicationComponent {
    unsafe { APPLICATION_COMPONENT.unwrap().as_ref() }
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let component: Box<dyn ApplicationComponent> = <dyn ApplicationComponent>::build();
    unsafe {
        APPLICATION_COMPONENT = Some(std::mem::transmute(&component));
    }
    let entry_point: &dyn AppEntryPoint = <dyn AppEntryPoint>::get(component.borrow());

    for listener in &entry_point.startup_listeners() {
        listener.on_startup();
    }

    let app = entry_point.app();
    app.initialize();
    app.start();
    Ok(())
}
lockjaw::epilogue!();
