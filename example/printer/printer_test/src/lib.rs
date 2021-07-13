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

use lockjaw::{epilogue, injectable, module, ComponentLifetime};
use printer::Printer;
use std::cell::{Ref, RefCell};

lockjaw::prologue!("src/lib.rs");

pub struct TestPrinter {
    pub messages: ::std::cell::RefCell<Vec<String>>,
}

#[injectable(scope: lockjaw::Singleton)]
impl TestPrinter {
    #[inject]
    pub fn new() -> TestPrinter {
        TestPrinter {
            messages: RefCell::default(),
        }
    }

    pub fn get_messages(&self) -> Ref<Vec<String>> {
        self.messages.borrow()
    }
}

impl Printer for TestPrinter {
    fn print(&self, message: &str) {
        self.messages.borrow_mut().push(message.to_owned())
    }
}

pub struct Module {}

#[module]
impl Module {
    #[binds]
    pub fn bind_printer(_impl: &crate::TestPrinter) -> ComponentLifetime<dyn ::printer::Printer> {}
}

epilogue!();
