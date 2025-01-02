/*
Copyright 2020 Google LLC

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

use std::cell::{RefCell, RefMut};

use lockjaw_common::manifest::{Component, Manifest};
use proc_macro2::Ident;
use quote::format_ident;

thread_local! {
    static MANIFEST :RefCell<Manifest> = RefCell::new(Manifest::new());
}

pub fn with_manifest<F, T>(f: F) -> T
where
    F: FnOnce(RefMut<Manifest>) -> T,
{
    MANIFEST.with(|m| {
        let manifest = m.borrow_mut();
        f(manifest)
    })
}

pub trait ProcessorComponent {
    fn impl_ident(&self) -> Ident;
}

impl ProcessorComponent for Component {
    fn impl_ident(&self) -> Ident {
        format_ident!("{}Impl", self.type_data.identifier_string())
    }
}
