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

//! Additional nested attributes for items under a [`#[module]`](crate::module)
//!
//! This mod is for documentation purpose only. All nested attributes should be unqualified (always
//! used as `#[attribute]` instead of `#[lockjaw::attribute]`).

#[doc = include_str ! ("provides.md")]
pub use lockjaw_processor::module_provides as provides;

#[doc = include_str ! ("binds.md")]
pub use lockjaw_processor::module_binds as binds;

#[doc = include_str ! ("binds_option_of.md")]
pub use lockjaw_processor::module_binds_option_of as binds_option_of;

#[doc = include_str ! ("multibinds.md")]
pub use lockjaw_processor::module_multibinds as multibinds;

#[doc = include_str ! ("into_vec.md")]
pub use lockjaw_processor::module_into_vec as into_vec;

#[doc = include_str ! ("elements_into_vec.md")]
pub use lockjaw_processor::module_elements_into_vec as elements_into_vec;

#[doc = include_str ! ("into_map.md")]
pub use lockjaw_processor::module_into_map as into_map;

#[doc = include_str ! ("qualified.md")]
pub use lockjaw_processor::module_qualified as qualified;
