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

//! Additional nested attributes for items under a [`#[injectable]`](crate::injectable)
//!
//! This mod is for documentation purpose only. All nested attributes should be unqualified (always
//! used as `#[attribute]` instead of `#[lockjaw::attribute]`).

#[doc = include_str ! ("inject.md")]
pub use lockjaw_processor::injectable_inject as inject;

#[doc = include_str ! ("factory.md")]
pub use lockjaw_processor::injectable_factory as factory;