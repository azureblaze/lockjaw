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

use lockjaw::{module, module_impl,component, injectable, root_epilogue, component_module_manifest};

#[injectable]
struct GreetCounter{
    counter : i32
}

impl GreetCounter{
    pub fn increment(&mut self) -> i32 {
        self.counter = self.counter + 1;
        self.counter
    }
}

pub trait Greeter {
    fn greet(&mut self) -> String;
}

#[injectable]
struct GreeterImpl {
    #[inject]
    greet_counter : crate::GreetCounter,
    #[inject]
    phrase : String
}

impl Greeter for GreeterImpl{
    fn greet(&mut self) -> String{
        format!("{} {}", self.phrase, self.greet_counter.increment())
    }
}

#[module]
struct MyModule {}

#[module_impl]
impl MyModule {
    #[binds]
    pub fn bind_greeter(_impl : crate::GreeterImpl) -> impl crate::Greeter {}

    #[provides]
    pub fn provide_string() -> String {
        "helloworld".to_owned()
    }
}

#[component_module_manifest]
struct ModuleManifest (crate::MyModule);

#[component(modules = "crate::ModuleManifest")]
trait MyComponent {
    fn greeter(&self) -> Box<dyn crate::Greeter>;
}

pub fn main() {
    let component: Box<dyn MyComponent> = MyComponent::new();
    let mut greeter = component.greeter();
    assert_eq!(greeter.greet(), "helloworld 1");
    assert_eq!(greeter.greet(), "helloworld 2");

    assert_eq!(component.greeter().greet(), "helloworld 1");
}

root_epilogue!("main.rs");
