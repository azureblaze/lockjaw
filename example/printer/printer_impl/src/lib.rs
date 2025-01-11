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

#[cfg(test)]
use lockjaw::{component, epilogue};
use lockjaw::{component_visible, injectable, module, Cl, Singleton};
use printer::Printer;

#[component_visible]
struct PrinterImpl {}

#[injectable]
impl PrinterImpl {
    #[inject]
    pub fn new() -> Self {
        PrinterImpl {}
    }
}

impl Printer for PrinterImpl {
    fn print(&self, message: &str) {
        println!("{}", message);
    }
}

#[component_visible]
struct Module {}

#[module(install_in: Singleton)]
impl Module {
    #[binds]
    pub fn bind_printer(_impl: crate::PrinterImpl) -> Cl<dyn ::printer::Printer> {}
}

#[cfg(test)]
#[component(modules: Module)]
pub trait TestComponent {
    fn printer(&'_ self) -> Cl<'_, dyn ::printer::Printer>;
}

#[test]
fn printer_impl_print() {
    let component = <dyn TestComponent>::new();
    component.printer().print("hello world");
}

#[cfg(test)]
epilogue!();
