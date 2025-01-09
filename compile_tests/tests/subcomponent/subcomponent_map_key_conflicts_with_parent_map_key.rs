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

struct BazModule {}

#[module]
impl BazModule {
    #[provides]
    #[into_map(i32_key: 1)]
    pub fn provide_i32() -> i32 {
        32
    }
}

#[subcomponent(modules: [BazModule])]
pub trait MySubcomponent<'a> {
    fn mi32(&self) -> HashMap<i32, i32>;
}

struct MyModule {}

#[module(subcomponents: [MySubcomponent])]
impl MyModule {
    #[provides]
    #[into_map(i32_key: 1)]
    pub fn provide_i32() -> i32 {
        32
    }
}

#[component(modules: [MyModule])]
pub trait MyComponent {
    fn sub(&'_ self) -> lockjaw::Cl<dyn MySubcomponentBuilder<'_>>;
}

lockjaw::epilogue!();
