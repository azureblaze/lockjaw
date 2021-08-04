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

use lockjaw::{
    component, injectable, Provider,
};

lockjaw::prologue!(
    "tests/provider_cyclic.rs"
);
struct Foo<'a> {
    bar: Provider<'a, Box<crate::Bar<'a>>>,
}

#[injectable]
impl<'a> Foo<'a> {
    #[inject]
    pub fn new(bar: Provider<'a, Box<crate::Bar<'a>>>) -> Foo<'a> {
        Self { bar }
    }
}

struct Bar<'a> {
    foo: Box<crate::Foo<'a>>,
}

#[injectable]
impl<'a> Bar<'a> {
    #[inject]
    pub fn new(foo: Box<crate::Foo<'a>>) -> Bar<'a> {
        Self { foo }
    }
}

#[component]
trait S {
    fn foo(&self) -> crate::Foo;
}

#[test]
fn main() {
    let component: Box<dyn S> = <dyn S>::build();
    component.foo().bar.get().foo.bar.get();
}

lockjaw::epilogue!(test);