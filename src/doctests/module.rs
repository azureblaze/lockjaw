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
/// #[module]
/// struct S {}
///
/// ```
mod module_non_impl {}

/// ```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
/// pub trait S {}
/// #[module]
/// impl dyn S {}
///
/// ```
mod module_not_path {}

/// ```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
///
/// struct S {}
///
/// #[module(foo="bar")]
/// impl S {}
/// ```
mod module_unknown_metadata {}

/// ```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
/// pub struct S {}
/// #[module]
/// impl S {
///     #[provides]
///     pub fn provide_string() {}
/// }
///
/// ```
mod provides_no_return_type {}

/// ```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
/// pub struct S {}
/// #[module]
/// impl S {
///     #[provides]
///     pub fn provide_string(self) -> String{"foo".to_owned()}
/// }
///
/// ```
mod provides_move_self {}

/// ```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
/// pub struct S {}
/// #[module]
/// impl S {
///     #[provides]
///     pub fn provide_string(_ : String) -> String{"foo".to_owned()}
/// }
///
/// ```
mod provides_param_not_identifier {}

/// ```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
/// pub struct S {}
///
/// pub trait ST{}
///
/// impl ST for S {}
/// #[module]
/// impl S {
///     #[binds]
///     pub fn bind_s() -> ComponentLifetime<dyn crate::ST>{}
/// }
///
/// ```
mod binds_no_return_type {}

/// ```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
/// pub struct S {}
///
/// pub trait ST{}
///
/// impl ST for S {}
///
/// #[module]
/// impl S {
///     #[binds]
///     pub fn bind_s() -> impl crate::ST{}
/// }
///
/// ```
mod binds_return_type_not_component_lifetime {}

/// ```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
/// pub struct S {}
///
/// pub trait ST{}
///
/// impl ST for S {}
/// #[module]
/// impl S {
///     #[binds]
///     pub fn bind_string() ->  ComponentLifetime<dyn crate::ST> {}
/// }
///
/// ```
mod binds_no_param {}

/// ```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
/// pub struct S {}
/// pub trait ST{}
/// impl ST for S {}
/// #[module]
/// impl S {
///     #[binds]
///     pub fn bind_string(a :i32, b:i32) -> ComponentLifetime<dyn crate::ST> {}
/// }
///
/// ```
mod binds_more_param {}

/// ```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
/// pub struct S {}
/// pub trait ST{}
/// impl ST for S {}
/// #[module]
/// impl S {
///     #[binds]
///     pub fn bind_string(self) -> ComponentLifetime<dyn crate::ST> {}
/// }
///
/// ```
mod binds_self {}

/// ```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
/// pub struct S {}
/// pub trait ST{}
/// impl ST for S {}
/// #[module]
/// impl S {
///     #[binds]
///     pub fn bind_string(_: i32) -> ComponentLifetime<dyn crate::ST> {}
/// }
///
/// ```
mod binds_no_identifier {}

/// ```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
/// use lockjaw::ComponentLifetime;
/// pub struct S {}
/// pub trait ST{}
/// impl ST for S {}
/// #[module]
/// impl S {
///     #[binds]
///     pub fn bind_s(&self) -> ComponentLifetime<dyn crate::ST> {
///         ComponentLifetime::Val(Box::new(S{}))
///     }
/// }
///
/// ```
mod binds_has_method_body {}
