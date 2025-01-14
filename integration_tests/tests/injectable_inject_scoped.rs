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
}

impl Foo {
    pub fn count(&self) -> u32 {
        let v: u32 = self.i.borrow().clone();
        self.i.replace(v + 1);
        v
    }
}

pub struct Bar<'a> {
    foo: &'a crate::Foo,
}

#[injectable]
impl Bar<'_> {
    #[inject]
    pub fn new(foo: &'_ crate::Foo) -> Bar<'_> {
        Bar { foo }
    }
}

#[component]
pub trait MyComponent {
    fn bar(&self) -> crate::Bar;
}
#[test]
pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    let bar1 = component.bar();
    let bar2 = component.bar();
    assert_eq!(bar1.foo.count(), 0);
    assert_eq!(bar2.foo.count(), 1);
}
epilogue!();
