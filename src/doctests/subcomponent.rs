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
/// ```compile_file
/// #[macro_use] extern crate lockjaw_processor;
/// lockjaw::prologue!("src/lib.rs");
/// struct BazModule {}
///
/// #[module]
/// impl BazModule {
///     #[provides]
///     #[into_vec]
///     pub fn provide_i32() -> i32 {
///         32
///     }
/// }
///
/// #[subcomponent(modules: [BazModule])]
/// pub trait MySubcomponent<'a> {
///     fn vi32(&self) -> Vec<i32>;
/// }
///
/// struct MyModule {}
///
/// #[module(subcomponents: [MySubcomponent])]
/// impl MyModule {
///
///     #[provides]
///     pub fn vi32() -> Vec<i32> {
///         Vec::new()
///     }
/// }
///
/// #[component(modules:  [MyModule])]
/// pub trait MyComponent {
///     fn sub(&'_ self) -> lockjaw::ComponentLifetime<dyn MySubcomponentBuilder<'_>>;
/// }
///
/// lockjaw::epilogue!(debug_output);
/// ```
mod subcomponent_multibinding_conflicts_with_parent_collection_binding {}
