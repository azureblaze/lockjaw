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
use lockjaw::{define_component, define_subcomponent, module, prologue, Cl};

prologue!("tests/sub_component_parent.rs");

struct Submodule {}

#[module(install_in: MySubcomponent)]
impl Submodule {
    #[provides]
    pub fn provide_i32() -> i32 {
        11
    }
}

#[define_subcomponent(parent: MyComponent)]
pub trait MySubcomponent<'a> {
    fn i32(&self) -> i32;
}

#[define_component]
pub trait MyComponent {
    fn sub(&'_ self) -> Cl<dyn MySubcomponentBuilder<'_>>;
}

#[test]
pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    let sub: Cl<dyn MySubcomponent> = component.sub().build();

    assert_eq!(sub.i32(), 11);
}

lockjaw::epilogue!();
