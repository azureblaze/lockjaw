/*
Copyright 2021 Google LLC

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

use lockjaw::{component, epilogue, injectable, module, prologue, Cl};

prologue!("src/main.rs");

trait I32Maker {
    fn i32(&self) -> i32;
}

struct I32MakerImpl;

#[injectable]
impl I32MakerImpl {
    #[inject]
    pub fn new() -> I32MakerImpl {
        I32MakerImpl {}
    }
}

impl I32Maker for I32MakerImpl {
    fn i32(&self) -> i32 {
        1337
    }
}

// ANCHOR: logger
pub trait Logger {
    fn log(&self, msg: &str);
}

struct StdoutLogger;

#[injectable]
impl StdoutLogger {
    #[inject]
    pub fn new() -> StdoutLogger {
        StdoutLogger
    }
}

impl Logger for StdoutLogger {
    fn log(&self, msg: &str) {
        println!("{}", msg);
    }
}
// ANCHOR_END: logger

// ANCHOR: greeter
pub struct Greeter<'component> {
    logger: Cl<'component, dyn Logger>,
}

#[injectable]
impl Greeter<'_> {
    #[inject]
    pub fn new(logger: Cl<dyn Logger>) -> Greeter {
        Greeter { logger }
    }

    pub fn greet(&self) {
        self.logger.log("helloworld!");
    }
}
// ANCHOR_END: greeter

struct MyModule;

#[module]
impl MyModule {
    // ANCHOR: provides_trait
    #[provides]
    pub fn provide_i32_maker(impl_: I32MakerImpl) -> Box<dyn I32Maker> {
        Box::new(impl_)
    }
    // ANCHOR_END: provides_trait

    // ANCHOR: binds
    #[binds]
    pub fn bind_stdout_logger(_impl: StdoutLogger) -> Cl<dyn Logger> {}
    // ANCHOR_END: binds
}

#[allow(unused)]
// ANCHOR: provides_component
#[component(modules: [MyModule])]
trait ProvideComponent {
    fn i32_maker(&'_ self) -> Box<dyn I32Maker + '_>;
}
// ANCHOR_END: provides_component

// ANCHOR: component
#[component(modules: [MyModule])]
trait MyComponent {
    fn greeter(&self) -> Greeter;
}
// ANCHOR_END: component

pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::build();

    component.greeter().greet();
}

// ANCHOR: test
#[cfg(test)]
pub mod testing {
    use crate::{Greeter, Logger};
    use lockjaw::{component, injectable, module, Cl};

    static mut MESSAGES: Vec<String> = Vec::new();

    pub struct TestLogger;

    #[injectable]
    impl TestLogger {
        #[inject]
        pub fn new() -> TestLogger {
            TestLogger
        }

        pub fn get_messages(&self) -> Vec<String> {
            unsafe { MESSAGES.clone() }
        }
    }

    impl Logger for TestLogger {
        fn log(&self, msg: &str) {
            unsafe { MESSAGES.push(msg.to_owned()) }
        }
    }

    pub struct TestModule;

    #[module]
    impl TestModule {
        #[binds]
        pub fn bind_test_logger(_impl: TestLogger) -> Cl<dyn Logger> {}
    }

    #[component(modules: [TestModule])]
    pub trait TestComponent {
        fn greeter(&self) -> Greeter;
        fn test_logger(&self) -> TestLogger;
    }

    #[test]
    fn test() {
        let component: Box<dyn TestComponent> = <dyn TestComponent>::build();

        component.greeter().greet();

        assert_eq!(
            component.test_logger().get_messages(),
            vec!["helloworld!".to_owned()]
        );
    }
}

// ANCHOR_END: test

epilogue!();
