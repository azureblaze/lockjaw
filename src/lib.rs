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

#![allow(stable_features)]
#![doc = include_str ! ("../README.md")]

mod build_script;
mod lazy;

#[doc = include_str ! ("component.md")]
pub use lockjaw_processor::component;

#[doc = include_str ! ("subcomponent.md")]
pub use lockjaw_processor::subcomponent;

#[doc = include_str ! ("define_component.md")]
pub use lockjaw_processor::define_component;

#[doc = include_str ! ("define_subcomponent.md")]
pub use lockjaw_processor::define_subcomponent;

pub mod component_attributes;

#[doc = include_str ! ("entry_point.md")]
pub use lockjaw_processor::entry_point;

#[doc = include_str ! ("builder_modules.md")]
pub use lockjaw_processor::builder_modules;

#[doc = include_str ! ("component_visible.md")]
pub use lockjaw_processor::component_visible;

#[doc = include_str ! ("epilogue.md")]
pub use lockjaw_processor::epilogue;

#[doc = include_str ! ("injectable.md")]
pub use lockjaw_processor::injectable;

pub mod injectable_attributes;

#[doc = include_str ! ("module.md")]
pub use lockjaw_processor::module;

pub mod module_attributes;

#[doc = include_str ! ("qualifier.md")]
pub use lockjaw_processor::qualifier;

#[doc(hidden)]
pub use lockjaw_processor::private_root_epilogue;
#[doc(hidden)]
pub use lockjaw_processor::private_test_epilogue;

mod component_lifetime;

pub use component_lifetime::Cl;

mod once;
pub use once::Once;

/// Function that must be called inside the
/// [cargo build script](https://doc.rust-lang.org/cargo/reference/build-scripts.html) to setup the
/// lockjaw environment.
///
/// lockjaw should be added to `[build-dependencies]` of the crate.
///
/// ```
/// // build.rs
/// fn main() {
///     lockjaw::build_script();
/// }
/// ```
pub fn build_script() {
    build_script::build_manifest()
}

mod provider;

pub use provider::Provider;

pub use lazy::Lazy;

#[doc = include_str ! ("singleton.md")]
pub trait Singleton {}
