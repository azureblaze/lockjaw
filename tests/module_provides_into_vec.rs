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

use lockjaw::{
    component, component_module_manifest, epilogue, injectable, module, ComponentLifetime,
};

pub use String as NamedString;

lockjaw::prologue!("tests/module_provides_into_vec.rs");

pub struct MyModule {}

pub trait Foo {
    fn foo(&self) -> String;
}

pub struct Bar {}

#[injectable]
impl Bar {
    #[inject]
    pub fn new() -> Self {
        Self {}
    }
}

impl Foo for Bar {
    fn foo(&self) -> String {
        "bar".to_owned()
    }
}

pub struct Baz {}

#[injectable]
impl Baz {
    #[inject]
    pub fn new() -> Self {
        Self {}
    }
}

impl Foo for Baz {
    fn foo(&self) -> String {
        "baz".to_owned()
    }
}

#[module]
impl MyModule {
    #[provides]
    pub fn provide_string() -> String {
        "string".to_owned()
    }

    #[provides]
    #[into_vec]
    pub fn provide_string1() -> String {
        "string1".to_owned()
    }

    #[provides]
    #[into_vec]
    pub fn provide_string2() -> String {
        "string2".to_owned()
    }

    #[provides]
    #[elements_into_vec]
    pub fn provide_strings() -> Vec<String> {
        vec!["string3".to_owned(), "string4".to_owned()]
    }

    #[binds]
    #[into_vec]
    pub fn bind_bar(impl_: crate::Bar) -> ComponentLifetime<dyn crate::Foo> {}

    #[binds]
    #[into_vec]
    pub fn bind_baz(impl_: crate::Baz) -> ComponentLifetime<dyn crate::Foo> {}
}

#[component_module_manifest]
pub struct MyModuleManifest {
    my_module: crate::MyModule,
}

#[component(modules = "crate::MyModuleManifest")]
pub trait MyComponent {
    fn string(&self) -> String;
    fn vec_string(&self) -> Vec<String>;

    fn vec_foo(&'_ self) -> Vec<ComponentLifetime<'_, dyn crate::Foo>>;
}

#[test]
pub fn into_vec() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    let v = component.vec_string();
    assert!(v.contains(&"string1".to_owned()));
    assert!(v.contains(&"string2".to_owned()));
    assert!(v.contains(&"string3".to_owned()));
    assert!(v.contains(&"string4".to_owned()));
}

#[test]
pub fn bind_into_vec() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    let v = component
        .vec_foo()
        .iter()
        .map(|foo| foo.foo())
        .collect::<Vec<String>>();
    assert!(v.contains(&"bar".to_owned()));
    assert!(v.contains(&"baz".to_owned()));
}

#[test]
pub fn regular_provision_not_affected() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    assert_eq!(component.string(), "string");
}

epilogue!(debug_output);