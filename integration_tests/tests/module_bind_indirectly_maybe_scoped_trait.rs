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

lockjaw::prologue!("tests/module_bind_indirectly_maybe_scoped_trait.rs");

pub struct Foo {}

#[injectable(scope: crate::MyComponent)]
impl Foo {
    #[inject]
    pub fn new() -> Self {
        Self {}
    }
}

pub trait MyTrait {
    fn hello(&self) -> String;
}

pub struct MyTraitImpl<'a> {
    foo: Cl<'a, crate::Foo>,
}

#[injectable]
impl MyTraitImpl<'_> {
    #[inject]
    pub fn new(foo: Cl<'_, crate::Foo>) -> MyTraitImpl {
        MyTraitImpl { foo }
    }
}

impl MyTrait for MyTraitImpl<'_> {
    fn hello(&self) -> String {
        "hello".to_owned()
    }
}

pub struct MyModule {}
#[module]
impl MyModule {
    #[binds]
    pub fn bind_my_trait(_impl: crate::MyTraitImpl) -> Cl<dyn crate::MyTrait> {}
}

#[component(modules: [MyModule])]
pub trait MyComponent {
    fn my_trait(&'_ self) -> Cl<'_, dyn crate::MyTrait>;
}
#[test]
pub fn main() {
    lockjaw_init();
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    assert_eq!(component.my_trait().hello(), "hello");
}
epilogue!();
