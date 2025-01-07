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

use lockjaw::{define_component, define_subcomponent, epilogue, module, Cl};

pub use String as NamedString;

pub struct MySubmodule {}

#[module(install_in: MySubcomponent)]
impl MySubmodule {
    #[provides]
    pub fn provide_string() -> String {
        "string".to_owned()
    }
}

#[define_subcomponent]
pub trait MySubcomponent<'a> {
    fn string(&self) -> String;
}

struct MyModule {}

#[module(install_in: MyComponent, subcomponents: [MySubcomponent])]
impl MyModule {}

#[define_component]
pub trait MyComponent {
    fn sub(&'_ self) -> Cl<dyn MySubcomponentBuilder<'_>>;
}

#[test]
pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    assert_eq!(component.sub().build().string(), "string");
}
epilogue!();
