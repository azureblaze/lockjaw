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

use lockjaw::{component, epilogue, module, qualifier};

use std::collections::HashMap;

lockjaw::prologue!("tests/module_multibinds.rs");

#[qualifier]
struct Q;

pub struct MyModule {}

// ANCHOR: multibinds
#[module]
impl MyModule {
    #[multibinds]
    fn vec_string() -> Vec<String> {}

    #[multibinds]
    #[qualified(Q)]
    fn q_vec_string() -> Vec<String> {}

    #[multibinds]
    fn map_string_string() -> HashMap<String, String> {}

    #[multibinds]
    #[qualified(Q)]
    fn q_map_string_string() -> HashMap<String, String> {}
}
// ANCHOR_END: multibinds

#[component(modules: [MyModule])]
pub trait MyComponent {
    fn vec_string(&self) -> Vec<String>;
    #[qualified(Q)]
    fn q_vec_string(&self) -> Vec<String>;

    fn map_string_string(&self) -> HashMap<String, String>;

    #[qualified(Q)]
    fn q_map_string_string(&self) -> HashMap<String, String>;
}

#[test]
pub fn multibinds_vec() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    let v = component.vec_string();
    assert!(v.is_empty());
}

#[test]
pub fn multibinds_qualified_vec() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    let v = component.q_vec_string();
    assert!(v.is_empty());
}

#[test]
pub fn multibinds_map() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    let v = component.map_string_string();
    assert!(v.is_empty());
}

#[test]
pub fn multibinds_qualified_map() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    let v = component.q_map_string_string();
    assert!(v.is_empty());
}

epilogue!();
