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
use lockjaw::{component, injectable, module, subcomponent, Cl};

pub struct A {
    i: i32,
}

#[injectable]
impl A {
    #[inject]
    pub fn new(i: i32) -> A {
        A { i }
    }
}

pub struct B {
    i: i32,
}

#[injectable]
impl B {
    #[inject]
    pub fn new(i: i32) -> B {
        B { i }
    }
}

struct Submodule {}

#[module]
impl Submodule {}

#[subcomponent(modules: [Submodule])]
pub trait MySubcomponent<'a> {
    fn a(&self) -> A;
    fn b(&self) -> B;
}

struct MyModule {}

#[module(subcomponents: [MySubcomponent])]
impl MyModule {
    #[provides]
    pub fn provide_i32() -> i32 {
        11
    }
}

#[component(modules: [MyModule])]
pub trait MyComponent {
    fn sub(&'_ self) -> Cl<dyn MySubcomponentBuilder<'_>>;
}

#[test]
pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    let sub: Cl<dyn MySubcomponent> = component.sub().build();

    assert_eq!(sub.a().i, sub.b().i);
}

lockjaw::epilogue!();
