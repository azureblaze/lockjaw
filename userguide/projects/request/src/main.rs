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

use lockjaw::{component, epilogue, injectable, prologue};

prologue!("src/main.rs");

struct Foo {}

#[injectable]
impl Foo {
    #[inject]
    pub fn new() -> Foo {
        Foo {}
    }
}

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

//ANCHOR: component
#[component]
trait MyComponent {
    fn create_foo(&self) -> Foo;

    fn create_bar(&self) -> Bar;
}
// ANCHOR_END: component

// ANCHOR: main
fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::build();

    let _foo = component.create_foo();
    let _bar = component.create_bar();
}
// ANCHOR_END: main

#[test]
fn test() {
    main()
}

epilogue!();
