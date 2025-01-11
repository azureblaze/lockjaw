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

use lockjaw_common::type_data::TypeData;

pub trait ProcessorTypeData {
    /// Full path of the type in local from (use crate:: within the same crate), which can be
    /// converted to tokens.
    ///
    /// Modifiers like & are omitted
    fn syn_type(&self) -> syn::Type;
    /// Unique identifier token representing the type.
    ///
    /// Modifiers like & are included.
    fn identifier(&self) -> syn::Ident;
}
impl ProcessorTypeData for TypeData {
    /// Full path of the type in local from (use crate:: within the same crate), which can be
    /// converted to tokens.
    ///
    /// Modifiers like & are omitted
    fn syn_type(&self) -> syn::Type {
        syn::parse_str(&self.local_string_path()).expect(&format!(
            "cannot parse type path {}",
            self.local_string_path()
        ))
    }

    /// Unique identifier token representing the type.
    ///
    /// Modifiers like & are included.
    fn identifier(&self) -> syn::Ident {
        quote::format_ident!("{}", self.identifier_string())
    }
}
