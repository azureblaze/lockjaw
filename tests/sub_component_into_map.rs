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
use lockjaw::{component, module, prologue, subcomponent, ComponentLifetime};
use std::collections::HashMap;

prologue!("tests/sub_component_into_map.rs");

struct BazModule {}

#[module]
impl BazModule {
    #[provides]
    #[into_map(i32_key: 1)]
    pub fn provide_i32() -> i32 {
        11
    }

    #[provides]
    #[into_map(i32_key: 1)]
    pub fn provide_i64() -> i64 {
        111
    }
}

#[subcomponent(modules: [BazModule])]
pub trait MySubcomponent<'a> {
    fn mi64(&self) -> HashMap<i32, i64>;
    fn mi32(&self) -> HashMap<i32, i32>;
}

struct MyModule {}

#[module(subcomponents: [MySubcomponent])]
impl MyModule {
    #[provides]
    #[into_map(i32_key: 2)]
    pub fn provide_i64() -> i64 {
        222
    }
}

#[component(modules: [MyModule])]
pub trait MyComponent {
    fn sub(&'_ self) -> ComponentLifetime<dyn MySubcomponentBuilder<'_>>;
}

#[test]
pub fn into_map_includes_parent() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    let sub: ComponentLifetime<dyn MySubcomponent> = component.sub().build();

    assert_eq!(*sub.mi64().get(&1).unwrap(), 111);
    assert_eq!(*sub.mi64().get(&2).unwrap(), 222);
}

#[test]
pub fn into_map_no_binding_in_parent() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    let sub: ComponentLifetime<dyn MySubcomponent> = component.sub().build();

    assert_eq!(*sub.mi32().get(&1).unwrap(), 11);
}

lockjaw::epilogue!(debug_output);
