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

struct Q;

pub struct MyModule {}

#[module]
impl MyModule {
    #[provides]
    #[qualified(crate::Q)]
    pub fn provide_q1_string() -> String {
        "q_string".to_owned()
    }
}

#[component(modules: MyModule)]
pub trait MyComponent {
    #[qualified(crate::Q)]
    fn string(&self) -> String;
}

#[test]
pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    component.string();
}
epilogue!();
lockjaw::epilogue!();
