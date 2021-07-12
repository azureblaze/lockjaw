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

#![allow(dead_code)]

use lockjaw::{component, epilogue, injectable, module, ComponentLifetime};

lockjaw::prologue!("tests/module_inject_param_autobox.rs");

pub struct Foo<'a> {
    bar: ComponentLifetime<'a, Bar>,
}

pub struct Bar {}

#[injectable]
impl Bar {
    #[inject]
    pub fn new() -> Self {
        Self {}
    }
}

pub struct MyModule {}
#[module]
impl MyModule {
    #[provides]
    pub fn provide_foo(bar: ComponentLifetime<'_, crate::Bar>) -> crate::Foo<'_> {
        Foo { bar }
    }
}

#[component(modules: [MyModule])]
pub trait MyComponent {
    fn foo(&self) -> crate::Foo;
}
#[test]
pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    component.foo();
}
epilogue!();
