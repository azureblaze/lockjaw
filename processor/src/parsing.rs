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
use crate::type_data::TypeData;
use proc_macro2::TokenStream;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::str::FromStr;
use syn::parse::Parser;
#[allow(unused_imports)] // somehow rust think this is unused.
use syn::spanned::Spanned;

pub fn is_attribute(syn_attr: &syn::Attribute, attr: &str) -> bool {
    get_attribute(syn_attr).eq(attr)
}

pub fn get_attribute(syn_attr: &syn::Attribute) -> String {
    if syn_attr.path.segments.len() != 1 {
        "".to_owned()
    } else {
        syn_attr
            .path
            .segments
            .first()
            .expect("missing segments")
            .ident
            .to_string()
    }
}

pub fn has_attribute(attrs: &Vec<syn::Attribute>, attr: &str) -> bool {
    attrs.iter().find(|a| is_attribute(a, attr)).is_some()
}

pub fn get_parenthesized_attribute_metadata(
    attr: TokenStream,
) -> Result<HashMap<String, String>, TokenStream> {
    if attr.is_empty() {
        return Ok(HashMap::new());
    }

    get_attribute_metadata(
        TokenStream::from_str(&strip_parentheses(&attr)?)
            .map_spanned_compile_error(attr.span(), "cannot parse string to tokens")?,
    )
}

pub fn get_parenthesized_type(attr: &TokenStream) -> Result<TypeData, TokenStream> {
    if attr.is_empty() {
        return spanned_compile_error(attr.span(), "path expected");
    }

    TypeData::from_str(&strip_parentheses(attr)?)
}

fn strip_parentheses(attr: &TokenStream) -> Result<String, TokenStream> {
    Ok(attr
        .to_string()
        .strip_prefix("(")
        .map_spanned_compile_error(attr.span(), "'(' expected at start")?
        .strip_suffix(")")
        .map_spanned_compile_error(attr.span(), "')' expected at end")?
        .to_owned())
}

/// Converts #[attr(key1="value1", key2="value2")] to key-value map.
pub fn get_attribute_metadata(attr: TokenStream) -> Result<HashMap<String, String>, TokenStream> {
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
pub fn get_types(types: Option<String>) -> Result<Vec<TypeData>, TokenStream> {
    if types.is_none() {
        return Ok(Vec::new());
    }
    types
        .unwrap()
        .split(",")
        .map(|path| -> syn::Path { syn::parse_str(&path).expect("cannot parse type string") })
        .map(|p| TypeData::from_path(p.borrow()))
        .collect()
}

fn to_string_literal(lit: &syn::Lit) -> Result<&syn::LitStr, TokenStream> {
    if let syn::Lit::Str(ref s) = lit {
        return Ok(s);
    }
    return spanned_compile_error(lit.span(), "string literal expected");
}
