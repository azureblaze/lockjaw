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
pub use String as NamedString;

pub struct MyModule {}

#[qualifier]
struct Q;

// ANCHOR: enum
#[derive(Eq, PartialEq, Hash)]
pub enum E {
    Foo,
    Bar,
}
// ANCHOR_END: enum

use E::Bar;

#[module]
impl MyModule {
    #[provides]
    pub fn provide_string() -> String {
        "string".to_owned()
    }

    // ANCHOR: string_key
    #[provides]
    #[into_map(string_key: "1")]
    pub fn provide_string1() -> String {
        "string1".to_owned()
    }

    #[provides]
    #[into_map(string_key: "2")]
    pub fn provide_string2() -> String {
        "string2".to_owned()
    }
    // ANCHOR_END: string_key

    // ANCHOR: qualified
    #[provides]
    #[qualified(Q)]
    #[into_map(string_key: "1")]
    pub fn provide_q_string1() -> String {
        "q_string1".to_owned()
    }
    // ANCHOR_END: qualified

    #[provides]
    #[qualified(Q)]
    #[into_map(string_key: "2")]
    pub fn provide_q_string2() -> String {
        "q_string2".to_owned()
    }

    // ANCHOR: i32_key
    #[provides]
    #[into_map(i32_key: 1)]
    pub fn provide_i32_string1() -> String {
        "string1".to_owned()
    }

    #[provides]
    #[into_map(i32_key: 2)]
    pub fn provide_i32_string2() -> String {
        "string2".to_owned()
    }
    // ANCHOR_END: i32_key

    // ANCHOR: enum_key
    #[provides]
    #[into_map(enum_key: E::Foo)]
    pub fn provide_enum_string1() -> String {
        "string1".to_owned()
    }

    #[provides]
    #[into_map(enum_key: Bar)]
    pub fn provide_enum_string2() -> String {
        "string2".to_owned()
    }
    // ANCHOR_END: enum_key
}

#[component(modules: [MyModule])]
pub trait MyComponent {
    fn string(&self) -> String;
    fn map_string(&self) -> HashMap<String, String>;
    #[qualified(Q)]
    fn q_map_string(&self) -> HashMap<String, String>;

    fn map_i32_string(&self) -> HashMap<i32, String>;
    fn map_enum_string(&self) -> HashMap<E, String>;
}

#[test]
pub fn into_map_string_key() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    let m = component.map_string();
    assert_eq!(m.get("1").unwrap(), "string1");
    assert_eq!(m.get("2").unwrap(), "string2");
}

#[test]
pub fn into_map_qualified() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    let m = component.q_map_string();
    assert_eq!(m.get("1").unwrap(), "q_string1");
    assert_eq!(m.get("2").unwrap(), "q_string2");
}

#[test]
pub fn into_map_i32_key() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    let m = component.map_i32_string();
    assert_eq!(m.get(&1).unwrap(), "string1");
    assert_eq!(m.get(&2).unwrap(), "string2");
}

#[test]
pub fn into_map_enum_key() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    let m = component.map_enum_string();
    assert_eq!(m.get(&E::Foo).unwrap(), "string1");
    assert_eq!(m.get(&Bar).unwrap(), "string2");
}

#[test]
pub fn regular_provision_not_affected() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    assert_eq!(component.string(), "string");
}

epilogue!();
