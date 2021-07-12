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

/// ```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
/// lockjaw::prologue!("src/lib.rs");
/// #[component]
/// struct S {}
/// ```
mod component_non_trait {}

/// ```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
/// lockjaw::prologue!("src/lib.rs");
/// #[component]
/// trait S {
///     fn foo();
/// }
/// ```
mod component_provision_no_return_type {}

/// ```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
/// lockjaw::prologue!("src/lib.rs");
/// trait Bar{}
/// #[component]
/// trait S {
///     fn foo() -> dyn Bar;
/// }
/// ```
mod component_provision_return_type_not_qualified {}

/// ```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
/// lockjaw::prologue!("src/lib.rs");
/// #[component(asdf)]
/// trait S {
/// }
/// ```
mod component_param_not_meta_name_value {}

/// ```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
/// lockjaw::prologue!("src/lib.rs");
/// #[component(foo::bar="baz")]
/// trait S {
/// }
/// ```
mod component_key_not_identifier {}

/// ```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
/// lockjaw::prologue!("src/lib.rs");
/// #[component(modules="crate::foo,crate::bar")]
/// trait S {
/// }
/// ```
mod component_modules_not_path {}

/// ```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
/// #[component(modules="foo")]
/// trait S {
/// }
/// ```
mod component_modules_path_not_qualified {}

///```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
/// lockjaw::prologue!("src/lib.rs");
/// pub trait Foo{}
/// #[component]
/// pub trait MyComponent {
///     fn foo(&self) -> Box<dyn crate::Foo>;
/// }
///
/// fn main(){}
/// ```
mod component_trait_object_provision_no_lifetime {}

///```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
/// lockjaw::prologue!("src/lib.rs");
///
/// use lockjaw::{component, injectable, epilogue};
///
/// pub struct Foo {}
///
/// #[injectable(scope : crate::MyComponent)]
/// impl Foo{
///     #[inject]
///     pub fn new() -> Self {
///         Self {}
///     }
/// }
///
/// #[component]
/// pub trait MyComponent {
///     fn foo(&self) -> crate::Foo;
/// }
///
/// pub fn main() {
///     let component: Box<dyn MyComponent> = MyComponent::new();
///     component.foo();
/// }
/// epilogue!();
///```
mod componenet_provision_non_scoped_of_scoped {}

///```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
/// lockjaw::prologue!("src/lib.rs");
///
/// #[lockjaw::component(foo="bar")]
/// pub trait MyComponent {
/// }

/// epilogue!();
///```
mod componenet_unknown_metadata {}
