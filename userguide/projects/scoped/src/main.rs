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

struct MyModule;

#[module]
impl MyModule {
    #[binds]
    pub fn bind_stdout_logger(_impl: StdoutLogger) -> Cl<dyn Logger> {}
}

#[component(modules: [MyModule])]
trait MyComponent {
    fn greeter(&self) -> Greeter;
}

pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::build();

    component.greeter().greet();
}

#[cfg(test)]
pub mod testing {
    use crate::{Greeter, Logger};
    use lockjaw::{component, injectable, module, Cl};
    use std::cell::RefCell;

    // ANCHOR: test_logger
    #[derive(Default)]
    pub struct TestLogger {
        messages: RefCell<Vec<String>>,
    }

    #[injectable(scope: TestComponent)]
    impl TestLogger {
        #[inject]
        pub fn new() -> TestLogger {
            TestLogger::default()
        }

        pub fn get_messages(&self) -> Vec<String> {
            self.messages.borrow().clone()
        }
    }

    impl Logger for TestLogger {
        fn log(&self, msg: &str) {
            self.messages.borrow_mut().push(msg.to_owned())
        }
    }

    // ANCHOR_END: test_logger

    pub struct TestModule;

    #[module]
    impl TestModule {
        // ANCHOR: binds
        #[binds]
        pub fn bind_test_logger(_impl: &TestLogger) -> Cl<dyn Logger> {}
        // ANCHOR_END: binds
    }

    #[component(modules: [TestModule])]
    pub trait TestComponent {
        fn greeter(&self) -> Greeter;
        // ANCHOR: component
        fn test_logger(&self) -> Cl<TestLogger>;
        // ANCHOR: component
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

epilogue!();
