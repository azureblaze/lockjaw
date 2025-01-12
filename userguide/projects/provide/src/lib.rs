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
use lockjaw::{component, epilogue, module};

// ANCHOR: module
struct MyModule;

#[module]
impl MyModule {
    // ANCHOR_END: module

    // ANCHOR: provides
    #[provides]
    pub fn provide_i32() -> i32 {
        42
    }
    // ANCHOR_END: provides

    // ANCHOR: provides_with_dep
    #[provides]
    pub fn provides_string(i: i32) -> String {
        format!("{}", i)
    }
    // ANCHOR_END: provides_with_dep
}

// ANCHOR: component
#[component(modules: [MyModule])]
trait MyComponent {
    fn i32(&self) -> i32;

    fn string(&self) -> String;
}
// ANCHOR_END: component

// ANCHOR: main
#[test]
fn test() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::build();

    assert_eq!(component.i32(), 42);
    assert_eq!(component.string(), "42");
}
// ANCHOR_END: main

epilogue!();
