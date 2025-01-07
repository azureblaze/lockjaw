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
extern crate lockjaw;

use lockjaw::{builder_modules, component, injectable, module, qualifier, subcomponent, Cl};

lockjaw::prologue!(
    "../../../compile_tests/tests/module/into_map_i32_key_collision.rs",
    "",
    "test"
);

pub struct MyModule {}

#[module]
impl MyModule {
    #[provides]
    #[into_map(i32_key: 1)]
    pub fn provide_string_1() -> String {
        "1".to_owned()
    }

    #[provides]
    #[into_map(i32_key: 1)]
    pub fn provide_string_2() -> String {
        "2".to_owned()
    }
}

#[component(modules: MyModule)]
pub trait MyComponent {
    fn i32_map(&self) -> std::collections::HashMap<i32, String>;
}

pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
}
lockjaw::epilogue!(test);
