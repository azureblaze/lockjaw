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

use crate::error::{spanned_compile_error, CompileError};
#[allow(unused)]
use crate::log;
use crate::manifests::type_from_path;
use crate::protos::manifest::Type;
use proc_macro2::TokenStream;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::str::FromStr;
use syn::parse::Parser;
#[allow(unused_imports)] // somehow rust think this is unused.
use syn::spanned::Spanned;

pub fn is_attribute(syn_attr: &syn::Attribute, attr: &str) -> bool {
    if syn_attr.path.segments.len() != 1 {
        false
    } else {
        syn_attr
            .path
            .segments
            .first()
            .expect("missing segments")
            .ident
            .to_string()
            .eq(attr)
    }
}

pub fn get_parenthesized_attributes(
    attr: TokenStream,
) -> Result<HashMap<String, String>, TokenStream> {
    if attr.is_empty() {
        return Ok(HashMap::new());
    }
    let s = attr
        .to_string()
        .strip_prefix("(")
        .map_spanned_compile_error(attr.span(), "'(' expected at start")?
        .strip_suffix(")")
        .map_spanned_compile_error(attr.span(), "')' expected at end")?
        .to_owned();

    get_attributes(
        TokenStream::from_str(&s)
            .map_spanned_compile_error(attr.span(), "cannot parse string to tokens")?,
    )
}

/// Converts #[attr(key1="value1", key2="value2")] to key-value map.
pub fn get_attributes(attr: TokenStream) -> Result<HashMap<String, String>, TokenStream> {
    let mut result = HashMap::new();
    let parser =
        syn::punctuated::Punctuated::<syn::MetaNameValue, syn::Token![,]>::parse_terminated;
    if attr.is_empty() {
        return Ok(result);
    }

    let metadata = parser
        .parse2(attr.clone())
        .map_spanned_compile_error(attr.span(), "MetaNameValue (key=\"value\", ...) expected")?;
    for data in metadata.iter() {
        result.insert(
            data.path
                .get_ident()
                .map_spanned_compile_error(data.path.span(), "path is not an identifier")?
                .to_string(),
            to_string_literal(&data.lit)?.value(),
        );
    }
    Ok(result)
}

/// Parses "foo::Bar, foo::Baz" to a list of types.
pub fn get_types(types: Option<String>) -> Result<Vec<Type>, TokenStream> {
    if types.is_none() {
        return Ok(Vec::new());
    }
    types
        .unwrap()
        .split(",")
        .map(|path| -> syn::Path { syn::parse_str(&path).expect("cannot parse type string") })
        .map(|p| type_from_path(p.borrow()))
        .collect()
}

fn to_string_literal(lit: &syn::Lit) -> Result<&syn::LitStr, TokenStream> {
    if let syn::Lit::Str(ref s) = lit {
        return Ok(s);
    }
    return spanned_compile_error(lit.span(), "string literal expected");
}
