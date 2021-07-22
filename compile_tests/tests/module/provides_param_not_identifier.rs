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
extern crate lockjaw;

use lockjaw::{
    builder_modules, component, injectable, module, qualifier, subcomponent, ComponentLifetime,
};

lockjaw::prologue!(
    "../../../compile_tests/tests/module/provides_param_not_identifier.rs",
    ""
);
pub struct S {}

#[module]
impl S {
    #[provides]
    pub fn provide_string(_: String) -> String {
        "foo".to_owned()
    }
}
lockjaw::epilogue!(test);
