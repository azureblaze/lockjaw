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

use lockjaw::{component, component_module_manifest, epilogue, module};

pub use String as NamedString;

pub struct MyModule {}

#[module]
impl MyModule {
    #[provides]
    pub fn provide_string() -> String {
        "string".to_owned()
    }

    #[binds_option_of]
    pub fn binds_option_of_string() -> String {}

    #[binds_option_of]
    pub fn binds_option_of_i32() -> i32 {}
}

#[component_module_manifest]
pub struct MyModuleManifest {
    my_module: crate::MyModule,
}

#[component(modules = "crate::MyModuleManifest")]
pub trait MyComponent {
    fn option_string(&self) -> Option<String>;
    fn option_i32(&self) -> Option<i32>;
}

#[test]
pub fn provided_value_returned() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    assert_eq!(component.option_string(), Some("string".to_owned()));
}

#[test]
pub fn not_provided_empty() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    assert_eq!(component.option_i32(), None);
}
epilogue!(debug_output);
