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

use lockjaw::{builder_modules, component, epilogue, module};

// ANCHOR: module
struct MyModule {
    i: i32,
}

#[module]
impl MyModule {
    #[provides]
    pub fn provide_i32(&self) -> i32 {
        self.i
    }
}
// ANCHOR_END: module

// ANCHOR: builder_modules
#[builder_modules]
struct MyBuilderModules {
    my_module: MyModule,
}
// ANCHOR_END: builder_modules

// ANCHOR: component
#[component(builder_modules: MyBuilderModules)]
trait MyComponent {
    fn i32(&self) -> i32;
}
// ANCHOR_END: component

// ANCHOR: main
#[test]
fn test() {
    let my_module = MyModule { i: 42 };
    let my_builder_modules = MyBuilderModules { my_module };
    let component: Box<dyn MyComponent> = <dyn MyComponent>::build(my_builder_modules);
    assert_eq!(component.i32(), 42);

    let other_component: Box<dyn MyComponent> = <dyn MyComponent>::build(MyBuilderModules {
        my_module: MyModule { i: 123 },
    });
    assert_eq!(other_component.i32(), 123);
}
// ANCHOR_END: main

epilogue!();
