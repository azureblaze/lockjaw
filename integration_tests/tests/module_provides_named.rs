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

use lockjaw::{component, epilogue, module};

pub use String as NamedString;

lockjaw::prologue!("tests/module_provides_named.rs");

pub struct MyModule {}

#[module]
impl MyModule {
    #[provides]
    pub fn provide_string() -> String {
        "string".to_owned()
    }

    #[provides]
    pub fn provide_named_string() -> crate::NamedString {
        "named_string".to_owned()
    }
}

#[component(modules: MyModule)]
pub trait MyComponent {
    fn string(&self) -> String;
    fn named_string(&self) -> crate::NamedString;
}
#[test]
pub fn main() {
    lockjaw_init();
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    assert_eq!(component.string(), "string");
    assert_eq!(component.named_string(), "named_string");
}
epilogue!();
