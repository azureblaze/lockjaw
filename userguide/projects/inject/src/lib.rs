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
#![allow(dead_code)]

use lockjaw::{epilogue, injectable};

// ANCHOR: foo
struct Foo {}

#[injectable]
impl Foo {
    #[inject]
    pub fn new() -> Foo {
        Foo {}
    }
}
// ANCHOR_END: foo

// ANCHOR: bar
struct Bar {
    foo: Foo,
    i: i32,
}

#[injectable]
impl Bar {
    #[inject]
    pub fn new(foo: Foo) -> Bar {
        Bar { foo, i: 42 }
    }
}
// ANCHOR_END: bar

//ANCHOR: factory
struct Factory {}

impl Factory {
    pub fn create_foo(&self) -> Foo {
        Foo::new()
    }

    pub fn create_bar(&self) -> Bar {
        Bar::new(self.create_foo())
    }
}

#[test]
fn test() {
    let factory = Factory {};

    let _foo = factory.create_foo();
    let _bar = factory.create_bar();
}
// ANCHOR_END: factory

epilogue!();
