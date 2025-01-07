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
use std::ops::Deref;

lockjaw::prologue!("tests/module_bind_scoped_trait.rs");

pub struct Foo {}

pub trait MyTrait {
    fn hello(&self) -> String;
}

pub struct MyTraitImpl {}

#[injectable(scope: crate::MyComponent)]
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

pub trait MyTrait2 {
    fn hello(&self) -> String;
}

pub struct MyTraitImpl2 {}

#[injectable]
impl MyTraitImpl2 {
    #[inject]
    pub fn new() -> Self {
        Self {}
    }
}

impl MyTrait2 for MyTraitImpl2 {
    fn hello(&self) -> String {
        "hello".to_owned()
    }
}

pub struct MyModule {}

#[module]
impl MyModule {
    #[binds]
    pub fn bind_my_trait(_impl: &crate::MyTraitImpl) -> Cl<dyn crate::MyTrait> {}

    #[binds(scope: MyComponent)]
    pub fn bind_my_trait2(_impl: &crate::MyTraitImpl2) -> Cl<dyn crate::MyTrait2> {}
}

#[component(modules: MyModule)]
pub trait MyComponent {
    fn my_trait(&'_ self) -> Cl<'_, dyn crate::MyTrait>;
    fn my_trait2(&'_ self) -> Cl<'_, dyn crate::MyTrait2>;
}

#[test]
pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    assert_eq!(component.my_trait().hello(), "hello");
    assert_eq!(component.my_trait2().hello(), "hello");
    assert_eq!(
        component.my_trait2().deref() as *const dyn MyTrait2,
        component.my_trait2().deref() as *const dyn MyTrait2
    );
}
epilogue!();
