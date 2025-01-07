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

use lockjaw::{builder_modules, component, epilogue, injectable, module};

struct MyModule {
    s: String,
}

#[module]
impl MyModule {
    #[provides]
    pub fn provide_string(&self) -> String {
        self.s.clone()
    }
}

#[builder_modules]
pub struct BuilderModules {
    my_module: MyModule,
}

#[derive(Debug)]
pub struct Foo {
    pub i: i32,
    pub s: String,
}

// ANCHOR: factory
#[injectable]
impl Foo {
    #[factory]
    fn create(#[runtime] i: i32, s: String) -> Self {
        Self { i, s }
    }
}
// ANCHOR_END: factory

#[component(builder_modules: BuilderModules)]
pub trait MyComponent {
    fn foo_factory(&self) -> FooFactory;
}

#[test]
pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::build(BuilderModules {
        my_module: MyModule {
            s: "helloworld".to_owned(),
        },
    });
    // ANCHOR: factory_use
    let foo = component.foo_factory().create(42);
    // ANCHOR_END: factory_use
    assert_eq!(foo.i, 42);
    assert_eq!(foo.s, "helloworld");
}

epilogue!();
