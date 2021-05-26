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

#![cfg_attr(nightly, feature(external_doc))]

#[cfg_attr(nightly, doc(include = "component.md"))]
pub use lockjaw_processor::component;

#[cfg_attr(nightly, doc(include = "component_module_manifest.md"))]
pub use lockjaw_processor::component_module_manifest;

#[cfg_attr(nightly, doc(include = "epilogue.md"))]
pub use lockjaw_processor::epilogue;

#[cfg_attr(nightly, doc(include = "injectable.md"))]
pub use lockjaw_processor::injectable;

#[cfg_attr(nightly, doc(include = "mod_epilogue.md"))]
pub use lockjaw_processor::mod_epilogue;

#[cfg_attr(nightly, doc(include = "module.md"))]
pub use lockjaw_processor::module;
#[doc(hidden)]
pub use lockjaw_processor::private_root_epilogue;
#[doc(hidden)]
pub use lockjaw_processor::private_test_epilogue;
#[doc(hidden)]
pub use lockjaw_processor::test_mod_epilogue;

mod doctests;

#[cfg(nightly)]
#[doc(include = "../README.md")]
pub mod readme {}

mod component_lifetime;
pub use component_lifetime::ComponentLifetime;

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
    // Do nothing. just forcing env var OUT_DIR to be set.
}

mod provider;

pub use provider::Provider;

mod lazy;

pub use lazy::Lazy;
