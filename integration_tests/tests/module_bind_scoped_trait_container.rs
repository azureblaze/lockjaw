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

use lockjaw::{component, epilogue, injectable, module, Cl};
use std::cell::RefCell;

pub trait MyTrait {
    fn hello(&self) -> String;
}

pub struct MyTraitImpl {}

#[injectable(scope: crate::MyComponent, container: RefCell)]
impl MyTraitImpl {
    #[inject]
    pub fn new() -> Self {
        Self {}
    }
}

impl MyTrait for MyTraitImpl {
    fn hello(&self) -> String {
        "hello".to_owned()
    }
}

pub struct MyModule {}

#[module]
impl MyModule {
    #[binds]
    pub fn bind_my_trait(_impl: &RefCell<crate::MyTraitImpl>) -> Cl<RefCell<dyn MyTrait>> {}
}

#[component(modules: MyModule)]
pub trait MyComponent {
    fn my_trait(&'_ self) -> Cl<'_, RefCell<dyn MyTrait>>;
}

#[test]
pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    assert_eq!(component.my_trait().borrow().hello(), "hello");
}
epilogue!();
