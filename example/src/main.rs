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

use lockjaw::{component, component_module_manifest, injectable, module, ComponentLifetime};

lockjaw::prologue!("src/main.rs");

pub struct Greeter<'a> {
    phrase: String,
    printer: ComponentLifetime<'a, dyn ::printer::Printer>,
}

#[injectable]
impl Greeter<'_> {
    #[inject]
    pub fn new<'a>(
        phrase: String,
        printer: ComponentLifetime<'a, dyn ::printer::Printer>,
    ) -> Greeter<'a> {
        Greeter { phrase, printer }
    }
    pub fn greet(&self) {
        self.printer.print(&self.phrase);
    }
}

struct MyModule {}

#[module]
impl MyModule {
    #[provides]
    pub fn provide_string() -> String {
        "helloworld".to_owned()
    }
}

#[component_module_manifest]
pub struct ModuleManifest(crate::MyModule, ::printer_impl::Module);

#[component(modules = "crate::ModuleManifest")]
pub trait MyComponent {
    fn greeter(&self) -> crate::Greeter;
}

pub fn main() {
    let component = <dyn MyComponent>::new();

    component.greeter().greet();
}

#[cfg(test)]
#[component_module_manifest]
pub struct TestModuleManifest(crate::MyModule, ::printer_test::Module);

#[cfg(test)]
#[component(modules = "crate::TestModuleManifest")]
pub trait TestComponent {
    fn greeter(&self) -> crate::Greeter;

    fn test_printer(&self) -> &::printer_test::TestPrinter;
}

#[test]
fn test_greeter() {
    let component = <dyn TestComponent>::new();
    component.greeter().greet();

    assert_eq!(
        component.test_printer().get_messages()[..],
        vec!["helloworld".to_owned()][..]
    );
}

lockjaw::epilogue!(debug_output);
