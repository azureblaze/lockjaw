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
///
/// struct Foo{}
/// #[component]
/// trait S {
///     fn foo() -> crate::Foo;
/// }
///
/// fn main(){}
///
/// epilogue!();
/// ```
mod graph_missing_binding {}

/// ```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
/// lockjaw::prologue!("src/lib.rs");
///
/// struct Foo{
///     bar: Box<crate::Bar>
/// }
/// #[injectable]
/// impl Foo{
///     #[inject]
///     pub fn new(bar: Box<crate::Bar>) -> Self {
///         Self { bar }
///     }
/// }
///
/// #[injectable]
/// struct Bar{
///     foo: Box<crate::Foo>
/// }
///
/// #[injectable]
/// impl Bar{
///     #[inject]
///     pub fn new(foo: Box<crate::Foo>) -> Self {
///         Self {foo}
///     }
/// }
///
/// #[component]
/// trait S {
///     fn foo() -> crate::Foo;
/// }
///
/// fn main(){}
///
/// epilogue!();
/// ```
mod graph_cyclic_dependency {}

/// ```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
/// lockjaw::prologue!("src/lib.rs");
///
/// struct Foo {}
///
/// impl Foo{
///     #[inject]
///     pub fn new() -> Self {
///         Self {}
///     }
/// }
///
/// struct M{}
///
/// #[module]
/// impl M {
///     #[provides]
///     pub fn provide_foo() -> crate::Foo {
///         Foo{}
///     }
/// }
///
/// #[component(modules=M)]
/// trait S {
///     fn foo() -> crate::Foo;
/// }
///
/// fn main(){}
///
/// epilogue!();
/// ```
mod graph_duplicated_binding {}

/// ```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
/// lockjaw::prologue!("src/lib.rs");
///
///
/// #[component(modules="crate::Mm")]
/// trait S {}
///
/// fn main(){}
///
/// epilogue!();
/// ```
mod graph_missing_module_manifest {}
