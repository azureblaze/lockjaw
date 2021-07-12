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
/// #[module]
/// struct S {}
///
/// ```
mod module_non_impl {}

/// ```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
/// lockjaw::prologue!("src/lib.rs");
/// pub trait S {}
/// #[module]
/// impl dyn S {}
///
/// ```
mod module_not_path {}

/// ```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
/// lockjaw::prologue!("src/lib.rs");
///
/// struct S {}
///
/// #[module(foo="bar")]
/// impl S {}
/// ```
mod module_unknown_metadata {}

/// ```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
/// lockjaw::prologue!("src/lib.rs");
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
/// lockjaw::prologue!("src/lib.rs");
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
/// lockjaw::prologue!("src/lib.rs");
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
/// lockjaw::prologue!("src/lib.rs");
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
/// lockjaw::prologue!("src/lib.rs");
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
/// lockjaw::prologue!("src/lib.rs");
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
/// lockjaw::prologue!("src/lib.rs");
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
/// lockjaw::prologue!("src/lib.rs");
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
/// lockjaw::prologue!("src/lib.rs");
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
/// lockjaw::prologue!("src/lib.rs");
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

/// ```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
/// lockjaw::prologue!("src/lib.rs");
/// pub struct S {}
/// #[module]
/// impl S {
///     #[provides]
///     #[binds]
///     pub fn provide_string(&self) -> String{"foo".to_owned()}
/// }
///
/// ```
mod provides_duplicated_bindings {}

/// ```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
/// lockjaw::prologue!("src/lib.rs");
/// use lockjaw::ComponentLifetime;
/// pub struct S {}
/// pub trait ST{}
/// impl ST for S {}
/// #[module]
/// impl S {
///     #[binds]
///     #[elements_into_vec]
///     pub fn bind_s(&self) -> Vec<ComponentLifetime<dyn crate::ST>> {
///         unimplemented!()
///     }
/// }
///
/// ```
mod binds_elements_into_vec {}

/// ```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
/// lockjaw::prologue!("src/lib.rs");
/// pub struct S {}
/// #[module]
/// impl S {
///     #[provides]
///     #[elements_into_vec]
///     pub fn provide_string(&self) -> String{"foo".to_owned()}
/// }
///
/// ```
mod provides_elements_into_vec_not_vec {}

/// ```compile_fail
///#[macro_use] extern crate lockjaw_processor;
/// lockjaw::prologue!("src/lib.rs");
///
/// struct Q;
///
///pub struct MyModule {}
///#[module]
///impl MyModule {
///    #[provides]
///    #[qualified(crate::Q)]
///    pub fn provide_q1_string() -> String {
///        "q_string".to_owned()
///    }
///}
///
///#[component(modules : MyModule)]
///pub trait MyComponent {
///    #[qualified(crate::Q)]
///    fn string(&self) -> String;
///}
///#[test]
///pub fn main() {
///    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
///    component.string();
///}
///epilogue!();
///
/// ```
mod qualifer_not_declared {}
