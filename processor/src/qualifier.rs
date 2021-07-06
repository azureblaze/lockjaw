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

use crate::error::{spanned_compile_error, CompileError};
use crate::manifest::with_manifest;
use crate::parsing;
use crate::prologue::{get_base_path, prologue_check};
use crate::type_data::TypeData;
use lazy_static::lazy_static;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashSet;
use syn::spanned::Spanned;

lazy_static! {
    static ref QUALIFIER_METADATA_KEYS: HashSet<String> = {
        let mut set = HashSet::<String>::new();
        set.insert("path".to_owned());
        set
    };
}

pub fn handle_qualifier_attribute(
    attr: TokenStream,
    input: TokenStream,
) -> Result<TokenStream, TokenStream> {
    let span = input.span();
    let item: syn::ItemStruct =
        syn::parse2(input).map_spanned_compile_error(span, "struct block expected")?;

    let attributes = parsing::get_attribute_metadata(attr.clone())?;
    for key in attributes.keys() {
        if !QUALIFIER_METADATA_KEYS.contains(key) {
            return spanned_compile_error(attr.span(), &format!("unknown key: {}", key));
        }
    }

    with_manifest(|mut manifest| {
        manifest.qualifiers.push(TypeData::from_local(
            &get_base_path(),
            &attributes.get("path").cloned(),
            &item.ident.to_string(),
        ))
    });

    let prologue_check = prologue_check();
    Ok(quote! {
        #item
        #prologue_check
    })
}
