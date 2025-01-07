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

#![allow(dead_code)]

use lockjaw::{component, epilogue, injectable, Lazy};

pub struct Counter {
    counter: ::std::cell::RefCell<i32>,
}

#[injectable(scope: crate::MyComponent)]
impl Counter {
    #[inject]
    pub fn new() -> Self {
        Self {
            counter: Default::default(),
        }
    }

    pub fn get(&self) -> i32 {
        self.counter.borrow().clone()
    }

    pub fn increment(&self) -> i32 {
        let mut value = self.counter.borrow_mut();
        *value = *value + 1;
        *value
    }
}

pub struct Foo<'a> {
    pub i: i32,
    counter: &'a Counter,
}

#[injectable(scope: crate::MyComponent)]
impl Foo<'_> {
    #[inject]
    pub fn new(counter: &'_ crate::Counter) -> Foo<'_> {
        Foo {
            i: counter.increment(),
            counter,
        }
    }
}

#[component]
pub trait MyComponent {
    fn foo(&self) -> Lazy<&crate::Foo>;

    fn counter(&self) -> &crate::Counter;
}

#[test]
pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();

    assert_eq!(component.foo().get().i, 1);
}

#[test]
pub fn before_get_not_created() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    let _foo = component.foo();

    assert_eq!(component.counter().get(), 0);
}

#[test]
pub fn multiple_get_same_instance() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    let lazy = component.foo();
    assert_eq!(lazy.get().i, 1);
    assert_eq!(lazy.get().i, 1);
}

epilogue!();
