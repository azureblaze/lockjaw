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

use lockjaw::{builder_modules, component, injectable, module, prologue, ComponentLifetime};
use printer::Printer;

prologue!("src/main.rs");

pub struct Greeter<'a> {
    phrase: String,
    printer: ComponentLifetime<'a, dyn Printer>,
}

#[injectable]
impl Greeter<'_> {
    #[inject]
    pub fn new<'a>(phrase: String, printer: ComponentLifetime<'a, dyn Printer>) -> Greeter<'a> {
        Greeter { phrase, printer }
    }
    pub fn greet(&self) {
        self.printer.print(&self.phrase);
    }
}

struct MyModule {
    phrase: String,
}

#[module]
impl MyModule {
    #[provides]
    pub fn provide_string(&self) -> String {
        self.phrase.clone()
    }

    #[provides]
    #[into_map(string_key: "1")]
    pub fn map_1(&self) -> String {
        "foo".to_owned()
    }

    #[provides]
    #[into_map(string_key: "2")]
    pub fn map_2(&self) -> String {
        "bar".to_owned()
    }
}

#[builder_modules]
pub struct BuilderModules {
    my_module: MyModule,
}

#[component(modules: [printer_impl::Module], builder_modules: BuilderModules)]
pub trait MyComponent {
    fn greeter(&self) -> Greeter;

    fn string_map(&self) -> HashMap<String, String>;
}

pub fn main() {
    let component = <dyn MyComponent>::build(BuilderModules {
        my_module: MyModule {
            phrase: "helloworld".to_owned(),
        },
    });

    component.greeter().greet();

    print!("{:#?}", component.string_map());
}

#[cfg(test)]
use printer_test::TestPrinter;
use std::collections::HashMap;

#[cfg(test)]
#[component(modules: [::printer_test::Module], builder_modules: BuilderModules)]
pub trait TestComponent {
    fn greeter(&self) -> Greeter;

    fn test_printer(&self) -> &TestPrinter;
}

#[test]
fn test_greeter() {
    let component = <dyn TestComponent>::build(BuilderModules {
        my_module: MyModule {
            phrase: "helloworld".to_owned(),
        },
    });
    component.greeter().greet();

    assert_eq!(
        component.test_printer().get_messages()[..],
        vec!["helloworld".to_owned()][..]
    );
}

lockjaw::epilogue!(debug_output);
