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
use crate::parsing;
use crate::type_data::TypeData;
use lazy_static::lazy_static;
use proc_macro2::TokenStream;
use quote::ToTokens;
use std::cell::RefCell;
use std::collections::HashSet;
use syn::spanned::Spanned;

struct LocalQualifier {
    identifier: String,
    additional_path: Option<String>,
}

thread_local! {
    static QUALIFIERS :RefCell<Vec<LocalQualifier>> = RefCell::new(Vec::new());
}

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

    let qualifier = LocalQualifier {
        identifier: item.ident.to_string(),
        additional_path: attributes.get("path").cloned(),
    };
    QUALIFIERS.with(|qualifiers| {
        qualifiers.borrow_mut().push(qualifier);
    });

    Ok(item.to_token_stream())
}

pub fn generate_manifest(base_path: &str) -> Vec<TypeData> {
    QUALIFIERS.with(|qualifiers| {
        let mut result = Vec::new();
        for local_qualifier in qualifiers.borrow().iter() {
            result.push(TypeData::from_local(
                base_path,
                &local_qualifier.additional_path,
                &local_qualifier.identifier,
            ));
        }
        qualifiers.borrow_mut().clear();
        result
    })
}
