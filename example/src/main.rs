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
    builder_modules, component_visible, define_component, entry_point, module, prologue, Cl,
    Singleton,
};
use printer::Printer;
#[cfg(test)]
use printer_test::TestPrinter;
use std::rc::Rc;

prologue!("src/main.rs");

#[component_visible]
struct Greeter<'a> {
    phrase: String,
    printer: Cl<'a, dyn Printer>,
}

#[lockjaw::injectable]
impl Greeter<'_> {
    #[inject]
    pub fn new<'a>(phrase: String, printer: Cl<'a, dyn Printer>) -> Greeter<'a> {
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
    pub fn bind_foo_creator(impl_: FooFactory) -> Cl<dyn FooCreator> {}
}

#[builder_modules]
struct BuilderModules {
    my_module: MyModule,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Foo {
    i: i32,
    phrase: String,
}

pub trait FooCreator {
    fn create(&self, i: i32) -> Foo;
}

#[lockjaw::injectable]
impl Foo {
    #[factory(implementing: FooCreator)]
    fn create(#[runtime] i: i32, phrase: String) -> Self {
        Self { i, phrase }
    }
}

#[component_visible]
struct RcI {}

#[lockjaw::injectable(scope: Singleton, container: Rc)]
impl RcI {
    #[inject]
    pub fn new() -> Self {
        Self {}
    }
}

#[entry_point(install_in: MyComponent)]
trait MyEntryPoint {
    fn greeter(&self) -> Greeter;

    fn foo_creator(&'_ self) -> Cl<'_, dyn FooCreator>;

    fn rci(&self) -> &Rc<RcI>;
}

#[define_component(builder_modules: BuilderModules)]
trait MyComponent {}

pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::build(BuilderModules {
        my_module: MyModule {
            phrase: "hello world".to_owned(),
        },
    });

    <dyn MyEntryPoint>::get(component.as_ref())
        .greeter()
        .greet();

    <dyn MyEntryPoint>::get(component.as_ref()).rci();
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

lockjaw::epilogue!(root, debug_output);
