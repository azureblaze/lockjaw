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

use lockjaw::{component, injectable};

pub struct Foo<'a> {
    bar: &'a Bar,
}

#[injectable(scope: crate::MyComponent)]
impl<'a> Foo<'a> {
    #[inject]
    pub fn new(bar: &'a crate::Bar) -> Self {
        Foo { bar }
    }
}

pub struct Bar {}

#[injectable(scope: crate::MyComponent)]
impl Bar {
    #[inject]
    pub fn new() -> Self {
        Bar {}
    }
}

#[component]
pub trait MyComponent {
    fn foo(&self) -> &crate::Foo;
    fn bar(&self) -> &crate::Bar;
}

#[test]
pub fn main() {
    let component = <dyn MyComponent>::new();
    component.foo();
    component.bar();
}

lockjaw::epilogue!();
