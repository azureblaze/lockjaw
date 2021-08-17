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

use lockjaw::{builder_modules, component, epilogue, injectable, module, Cl};

lockjaw::prologue!("tests/injectable_factory.rs");

struct MyModule {
    phrase: String,
}

#[module]
impl MyModule {
    #[provides]
    pub fn provide_string(&self) -> String {
        self.phrase.clone()
    }

    #[binds]
    pub fn bind_foo_creator(impl_: FooFactory) -> Cl<dyn FooCreator> {}
}

#[builder_modules]
pub struct BuilderModules {
    my_module: MyModule,
}

pub struct Foo {
    pub i: i32,
    pub phrase: String,
}

pub trait FooCreator {
    fn create(&self, i: i32) -> Foo;
}

#[injectable]
impl Foo {
    #[factory(implementing: FooCreator)]
    fn create(#[runtime] i: i32, phrase: String) -> Self {
        Self { i, phrase }
    }
}

#[component(builder_modules: BuilderModules)]
pub trait MyComponent {
    fn foo_creator(&self) -> Cl<dyn FooCreator>;
}

#[test]
pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::build(BuilderModules {
        my_module: MyModule {
            phrase: "helloworld".to_owned(),
        },
    });

    let foo = component.foo_creator().create(42);

    assert_eq!(foo.i, 42);
    assert_eq!(foo.phrase, "helloworld");
}

epilogue!();
