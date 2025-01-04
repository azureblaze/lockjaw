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
use std::cell::RefCell;
use std::marker::PhantomData;

lockjaw::prologue!("tests/injectable_scoped_container_lifetimed.rs");

pub struct Foo<'a> {
    pub i: u32,
    p: PhantomData<&'a i32>,
}

#[injectable(scope: crate::MyComponent, container: RefCell)]
impl Foo<'_> {
    #[inject]
    pub fn new() -> Self {
        Self {
            i: Default::default(),
            p: PhantomData::default(),
        }
    }

    pub fn count(&mut self) -> u32 {
        self.i = self.i + 1;
        self.i
    }
}

#[component]
pub trait MyComponent {
    fn foo(&self) -> &RefCell<crate::Foo>;
}

#[test]
pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    let foo1 = component.foo();
    let foo2 = component.foo();

    assert_eq!(foo1.borrow_mut().count(), 1);
    assert_eq!(foo1.borrow_mut().count(), 2);
    assert_eq!(foo2.borrow_mut().count(), 3);
}
epilogue!();
