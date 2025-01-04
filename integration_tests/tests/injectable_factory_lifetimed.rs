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

use lockjaw::{component, epilogue, injectable, module};

lockjaw::prologue!("tests/injectable_factory_lifetimed.rs");

struct MyModule {}

#[module]
impl MyModule {
    #[provides(scope: MyComponent)]
    pub fn provide_string(&self) -> String {
        "foo".to_owned()
    }
}

#[derive(Debug)]
pub struct Foo<'a> {
    pub i: i32,
    pub phrase: &'a String,
}

#[injectable]
impl Foo<'_> {
    #[factory]
    fn create<'a>(#[runtime] i: i32, phrase: &'a String) -> Foo<'a> {
        Foo { i, phrase }
    }
}

#[component(modules: MyModule)]
pub trait MyComponent {
    fn foo_factory(&self) -> FooFactory;
}

#[test]
pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::build();

    let foo = component.foo_factory().create(42);

    assert_eq!(foo.i, 42);
    assert_eq!(foo.phrase, "foo");
}

epilogue!();
