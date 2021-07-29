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
    builder_modules, component_visible, define_component, entry_point, injectable, module,
    prologue, ComponentLifetime,
};
use printer::Printer;
#[cfg(test)]
use printer_test::TestPrinter;

prologue!("src/main.rs");

#[component_visible]
struct Greeter<'a> {
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

    #[binds]
    pub fn bind_foo_creator(impl_: FooFactory) -> ComponentLifetime<dyn FooCreator> {}
}

#[builder_modules]
struct BuilderModules {
    my_module: MyModule,
}

#[derive(Debug)]
pub struct Foo {
    i: i32,
    phrase: String,
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

#[entry_point(install_in: MyComponent)]
trait MyEntryPoint {
    fn greeter(&self) -> Greeter;

    fn foo_creator(&'_ self) -> ComponentLifetime<'_, dyn FooCreator>;
}

#[define_component(builder_modules: BuilderModules)]
trait MyComponent {}

pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::build(BuilderModules {
        my_module: MyModule {
            phrase: "helloworld".to_owned(),
        },
    });

    <dyn MyEntryPoint>::get(component.as_ref())
        .greeter()
        .greet();
}

#[cfg(test)]
#[define_component(builder_modules: BuilderModules)]
trait TestComponent {
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

lockjaw::epilogue!(root debug_output);
