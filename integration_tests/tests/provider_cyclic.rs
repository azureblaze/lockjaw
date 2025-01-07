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

#![allow(dead_code)]

use lockjaw::{component, injectable, Provider};

lockjaw::prologue!("tests/provider_cyclic.rs");

// ANCHOR: cyclic
struct Foo<'component> {
    bar_provider: Provider<'component, Box<Bar<'component>>>,
}

#[injectable]
impl<'component> Foo<'component> {
    #[inject]
    pub fn new(bar: Provider<'component, Box<Bar<'component>>>) -> Foo<'component> {
        Self { bar_provider: bar }
    }

    pub fn create_bar(&self) -> Box<Bar<'component>> {
        self.bar_provider.get()
    }
}

struct Bar<'component> {
    foo: Box<Foo<'component>>,
}

#[injectable]
impl<'component> Bar<'component> {
    #[inject]
    pub fn new(foo: Box<Foo<'component>>) -> Bar<'component> {
        Self { foo }
    }
}
// ANCHOR_END: cyclic

#[component]
trait S {
    fn foo(&self) -> Foo;
}

#[test]
fn main() {
    let component: Box<dyn S> = <dyn S>::build();
    component.foo().create_bar().foo.create_bar();
}

lockjaw::epilogue!(test);
