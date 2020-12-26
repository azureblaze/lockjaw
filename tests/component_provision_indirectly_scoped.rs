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

use lockjaw::{
    component, component_module_manifest, injectable, module, module_impl, test_epilogue,
    MaybeScoped,
};

#[injectable(scope = "crate::MyComponent")]
struct GreetCounter {
    counter: ::std::cell::RefCell<i32>,
}

impl GreetCounter {
    pub fn increment(&self) -> i32 {
        let mut value = self.counter.borrow_mut();
        *value = *value + 1;
        *value
    }
}

pub trait Greeter {
    fn greet(&self) -> String;
}

#[injectable]
struct GreeterImpl<'a> {
    #[inject]
    counter: &'a crate::GreetCounter,
    #[inject]
    phrase: String,
}

impl Greeter for GreeterImpl<'_> {
    fn greet(&self) -> String {
        format!("{} {}", self.phrase, self.counter.increment())
    }
}

#[module]
struct MyModule {}

#[module_impl]
impl MyModule {
    #[binds]
    pub fn bind_greeter(_impl: crate::GreeterImpl) -> impl crate::Greeter {}

    #[provides]
    pub fn provide_string() -> String {
        "helloworld".to_owned()
    }
}

#[component_module_manifest]
pub struct ModuleManifest(crate::MyModule);

#[component(modules = "crate::ModuleManifest")]
pub trait MyComponent {
    fn greeter(&'_ self) -> MaybeScoped<'_, dyn crate::Greeter>;
}

pub fn main() {
    let component = MyComponent::new();

    assert_eq!(component.greeter().greet(), "helloworld 1");
    assert_eq!(component.greeter().greet(), "helloworld 2");
}

test_epilogue!();
