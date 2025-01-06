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
use lockjaw::{component, define_subcomponent, entry_point, module, prologue, Cl};
use std::ops::Deref;

prologue!("tests/sub_component_into_vec.rs");

struct BazModule {}

#[module]
impl BazModule {
    #[provides]
    #[into_vec]
    pub fn provide_i32() -> i32 {
        32
    }
}

#[define_subcomponent(modules: [BazModule])]
pub trait MySubcomponent<'a> {}

#[entry_point(install_in: MySubcomponent)]
pub trait MyEntryPoint {
    fn vi32(&self) -> Vec<i32>;
}

struct MyModule {}

#[module(subcomponents: [MySubcomponent])]
impl MyModule {}

#[component(modules: [MyModule])]
pub trait MyComponent {
    fn sub(&'_ self) -> Cl<dyn MySubcomponentBuilder<'_>>;
}

#[test]
pub fn main() {
    lockjaw_init();
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    let sub: Cl<dyn MySubcomponent> = component.sub().build();

    assert!(<dyn MyEntryPoint>::get(sub.deref()).vi32().contains(&32));
}

lockjaw::epilogue!(debug_output);
