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
    builder_modules, component, entry_point, injectable, module, qualifier, subcomponent,
    ComponentLifetime,
};

lockjaw::prologue!(
    "../../../compile_tests/tests/component/entry_point_not_installed_in_defined_component.rs",
    ""
);

#[component]
pub trait MyComponent {}

#[entry_point(install_in: MyComponent)]
pub trait MyEntryPoint {}

lockjaw::epilogue!(root test);
