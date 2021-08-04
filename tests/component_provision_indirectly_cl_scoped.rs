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

use lockjaw::{component, epilogue, injectable, module, Cl};
use std::ops::Deref;

lockjaw::prologue!("tests/component_provision_indirectly_cl_scoped.rs");

struct GreetCounter {
    counter: ::std::cell::RefCell<i32>,
}

#[injectable(scope: crate::MyComponent)]
impl GreetCounter {
    #[inject]
    pub fn new() -> Self {
        Self {
            counter: Default::default(),
        }
    }

    pub fn increment(&self) -> i32 {
        let mut value = self.counter.borrow_mut();
        *value = *value + 1;
        *value
    }
}

pub trait Greeter {
    fn greet(&self) -> String;
}

struct GreeterImpl<'a> {
    counter: Cl<'a, crate::GreetCounter>,
    phrase: String,
}

#[injectable]
impl GreeterImpl<'_> {
    #[inject]
    pub fn new(counter: Cl<'_, crate::GreetCounter>, phrase: String) -> GreeterImpl<'_> {
        GreeterImpl { counter, phrase }
    }
}

impl Greeter for GreeterImpl<'_> {
    fn greet(&self) -> String {
        format!("{} {}", self.phrase, self.counter.increment())
    }
}

struct MyModule {}

#[module]
impl MyModule {
    #[binds]
    pub fn bind_greeter(_impl: crate::GreeterImpl) -> Cl<dyn crate::Greeter> {}

    #[provides]
    pub fn provide_string() -> String {
        "helloworld".to_owned()
    }
}

#[component(modules: [MyModule])]
pub trait MyComponent {
    fn greeter(&'_ self) -> Cl<'_, dyn crate::Greeter>;

    fn string(&self) -> Cl<String>;
}

#[test]
pub fn main() {
    let component = <dyn MyComponent>::new();

    assert_eq!(component.greeter().greet(), "helloworld 1");
    assert_eq!(component.greeter().greet(), "helloworld 2");
}

#[test]
pub fn regular_binding() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();

    assert_eq!(component.string().deref(), "helloworld");
}

epilogue!(debug_output);
