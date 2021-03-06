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
/// #[lockjaw::injectable]
/// trait S {}
/// ```
mod injectable_non_struct {}

/// ```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
/// lockjaw::prologue!("src/lib.rs");
/// #[lockjaw::injectable]
/// struct S(String);
/// ```
mod injectable_tuple {}

///```compile_fail
/// use lockjaw::{component, injectable, epilogue};
/// lockjaw::prologue!("src/lib.rs");
///
/// pub struct Foo {}
///
/// #[injectable]
/// impl Foo {
///     pub fn new() -> Self {
///         Self {}
///     }
/// }
/// epilogue!();
///```
mod injectable_no_inject {}

///```compile_fail
/// use lockjaw::{component, injectable, epilogue};
/// lockjaw::prologue!("src/lib.rs");
///
/// pub struct Foo {}
///
/// #[injectable]
/// impl Foo {
///     #[inject]
///     pub fn new() -> Self {
///         Self {}
///     }
///     #[inject]
///     pub fn new2() -> Self {
///         Self {}
///     }
/// }
/// epilogue!();
///```
mod injectable_multiple_inject {}

/// ```compile_fail
/// #[macro_use] extern crate lockjaw_processor;
/// lockjaw::prologue!("src/lib.rs");
/// #[lockjaw::injectable(foo="bar")]
/// struct S {};
/// ```
mod injectable_unknown_metadata {}
