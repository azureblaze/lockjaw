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

use lockjaw::{component, epilogue, module, qualifier};

pub use String as NamedString;

lockjaw::prologue!("tests/module_provides_qualifier.rs");

//ANCHOR: decl
#[qualifier]
pub struct Q1;

//ANCHOR_END: decl
#[qualifier]
pub struct Q2;

pub struct MyModule {}

//ANCHOR: module
#[module]
impl MyModule {
    #[provides]
    pub fn provide_string() -> String {
        "string".to_owned()
    }

    #[provides]
    #[qualified(Q1)]
    pub fn provide_q1_string() -> String {
        "q1_string".to_owned()
    }

    #[provides]
    #[qualified(Q2)]
    pub fn provide_q2_string() -> String {
        "q2_string".to_owned()
    }
}
// ANCHOR_END: module

//ANCHOR: component
#[component(modules: [MyModule])]
pub trait MyComponent {
    fn string(&self) -> String;
    #[qualified(Q1)]
    fn q1_string(&self) -> String;
    #[qualified(Q2)]
    fn q2_string(&self) -> String;
}
//ANCHOR_END: component

#[test]
pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    assert_eq!(component.string(), "string");
    assert_eq!(component.q1_string(), "q1_string");
    assert_eq!(component.q2_string(), "q2_string");
}
epilogue!();
