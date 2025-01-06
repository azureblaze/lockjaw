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

use crate::manifest::Manifest;
use crate::manifest_parser::Mod;
use anyhow::{Context, Result};
use proc_macro2::TokenStream;

pub fn handle_qualifier_attribute(
    _attr: TokenStream,
    input: TokenStream,
    mod_: &Mod,
) -> Result<Manifest> {
    let item: syn::ItemStruct = syn::parse2(input).with_context(|| "struct block expected")?;

    let mut manifest = Manifest::new();
    manifest
        .qualifiers
        .push(crate::type_data::from_local(&item.ident.to_string(), mod_)?);
    Ok(manifest)
}
