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

use lockjaw::{component, epilogue, injectable};

lockjaw::prologue!("tests/injectable_scoped.rs");

pub struct Foo {
    pub i: ::std::cell::RefCell<u32>,
}

#[injectable(scope: crate::MyComponent)]
impl Foo {
    #[inject]
    pub fn new() -> Self {
        Self {
            i: Default::default(),
        }
    }

    pub fn count(&self) -> u32 {
        let v: u32 = self.i.borrow().clone();
        self.i.replace(v + 1);
        v
    }
}

#[component]
pub trait MyComponent {
    fn foo(&self) -> &crate::Foo;
}
#[test]
pub fn main() {
    let component: Box<dyn MyComponent> = MyComponentBuilder {}.build();
    let foo1 = component.foo();
    let foo2 = component.foo();

    assert_eq!(foo1.count(), 0);
    assert_eq!(foo1.count(), 1);
    assert_eq!(foo2.count(), 2);
}
epilogue!();
