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

use lockjaw::{entry_point, module, prologue};
use test_dep::DepDefinedComponent;

prologue!("tests/define_component.rs");

struct MyModule {}

#[module(install_in: DepDefinedComponent)]
impl MyModule {
    #[provides]
    pub fn provide_i(&self) -> i32 {
        42
    }
}

#[entry_point(install_in: DepDefinedComponent)]
pub trait MyEntryPoint {
    fn i(&self) -> i32;
}

#[test]
pub fn main() {
    let component: Box<dyn DepDefinedComponent> = <dyn DepDefinedComponent>::new();

    assert_eq!(<dyn MyEntryPoint>::get(component.as_ref()).i(), 42)
}

lockjaw::epilogue!(root);
