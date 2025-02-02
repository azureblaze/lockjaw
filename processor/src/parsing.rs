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
use proc_macro2::{Span, TokenStream};
use std::collections::HashMap;
use syn::parse::Parser;
#[allow(unused_imports)] // somehow rust think this is unused.
use syn::spanned::Spanned;
use syn::Meta;

pub fn is_attribute(syn_attr: &syn::Attribute, attr: &str) -> bool {
    get_attribute(syn_attr).eq(attr)
}

pub fn get_attribute(syn_attr: &syn::Attribute) -> String {
    if syn_attr.meta.path().segments.len() != 1 {
        "".to_owned()
    } else {
        syn_attr
            .meta
            .path()
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

pub fn get_parenthesized_field_values(
    meta: &Meta,
) -> Result<HashMap<String, FieldValue>, TokenStream> {
    match meta {
        Meta::Path(_) => Ok(HashMap::new()),
        Meta::List(list) => get_attribute_field_values(list.tokens.clone()),
        Meta::NameValue(_) => {
            panic!("list expected")
        }
    }
}

pub fn get_path(attr: &TokenStream) -> Result<syn::Path, TokenStream> {
    if attr.is_empty() {
        return spanned_compile_error(attr.span(), "path expected");
    }
    syn::parse2(attr.clone()).map_spanned_compile_error(attr.span(), "path expected")
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum FieldValue {
    StringLiteral(Span, String),
    IntLiteral(Span, i64),
    FloatLiteral(Span, f64),
    BoolLiteral(Span, bool),
    Path(Span, syn::Path),
    Array(Span, Vec<FieldValue>),
    FieldValues(Span, HashMap<String, FieldValue>),
}

impl FieldValue {
    pub fn span(&self) -> Span {
        match self {
            FieldValue::StringLiteral(ref span, _) => span.clone(),
            FieldValue::IntLiteral(ref span, _) => span.clone(),
            FieldValue::FloatLiteral(ref span, _) => span.clone(),
            FieldValue::BoolLiteral(ref span, _) => span.clone(),
            FieldValue::Path(ref span, _) => span.clone(),
            FieldValue::Array(ref span, _) => span.clone(),
            FieldValue::FieldValues(ref span, _) => span.clone(),
        }
    }

    pub fn get_paths(&self) -> Result<Vec<(syn::Path, Span)>, TokenStream> {
        match self {
            FieldValue::Path(ref span, ref path) => Ok(vec![(path.clone(), span.clone())]),
            FieldValue::Array(_, ref array) => array
                .iter()
                .map(|f| {
                    if let FieldValue::Path(ref span, ref path) = f {
                        Ok((path.clone(), span.clone()))
                    } else {
                        spanned_compile_error(self.span(), "path expected")
                    }
                })
                .collect(),
            _ => spanned_compile_error(self.span(), "path expected"),
        }
    }
}

/// Converts #[attr(key1 : "value1", key2 : value2)] to key-value map.
pub fn get_attribute_field_values(
    attr: TokenStream,
) -> Result<HashMap<String, FieldValue>, TokenStream> {
    let parser = syn::punctuated::Punctuated::<syn::FieldValue, syn::Token![,]>::parse_terminated;
    if attr.is_empty() {
        return Ok(HashMap::new());
    }

    let field_values = parser
        .parse2(attr.clone())
        .map_spanned_compile_error(attr.span(), "FieldValue (key: value, ...) expected")?;

    parse_punctuated_field_values(&field_values)
}

fn parse_punctuated_field_values(
    field_values: &syn::punctuated::Punctuated<syn::FieldValue, syn::Token![,]>,
) -> Result<HashMap<String, FieldValue>, TokenStream> {
    let mut result = HashMap::new();
    for field in field_values.iter() {
        if let syn::Member::Named(ref name) = field.member {
            result.insert(
                name.to_string(),
                parse_field_value(&field.expr, field.span())?,
            );
        } else {
            return spanned_compile_error(field.span(), "field should have named member");
        }
    }
    Ok(result)
}

fn parse_field_value(expr: &syn::Expr, span: Span) -> Result<FieldValue, TokenStream> {
    match expr {
        syn::Expr::Lit(ref lit) => match lit.lit {
            syn::Lit::Str(ref str_) => Ok(FieldValue::StringLiteral(str_.span(), str_.value())),
            syn::Lit::Bool(ref bool_) => Ok(FieldValue::BoolLiteral(bool_.span(), bool_.value())),
            syn::Lit::Int(ref int) => Ok(FieldValue::IntLiteral(
                int.span(),
                int.base10_parse::<i64>()
                    .map_spanned_compile_error(int.span(), "unable to parse integer to i64")?,
            )),
            syn::Lit::Float(ref float) => Ok(FieldValue::FloatLiteral(
                float.span(),
                float
                    .base10_parse::<f64>()
                    .map_spanned_compile_error(float.span(), "unable to parse integer to f64")?,
            )),
            _ => spanned_compile_error(span, &format!("unable to handle literal value {:?}", lit)),
        },
        syn::Expr::Path(ref path) => Ok(FieldValue::Path(span, path.path.clone())),
        syn::Expr::Array(ref array) => {
            let mut values: Vec<FieldValue> = Vec::new();
            for expr in &array.elems {
                values.push(parse_field_value(expr, expr.span())?);
            }
            Ok(FieldValue::Array(span, values))
        }
        syn::Expr::Struct(ref struct_) => Ok(FieldValue::FieldValues(
            span,
            parse_punctuated_field_values(&struct_.fields)?,
        )),
        _ => spanned_compile_error(span, &format!("invalid field value {:?}", expr)),
    }
}

pub fn type_string(ty: &syn::Type) -> Result<String, TokenStream> {
    if let syn::Type::Path(ref path) = ty {
        let segments: Vec<String> = path
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect();
        Ok(segments.join("_"))
    } else {
        spanned_compile_error(ty.span(), &"path expected".to_string())
    }
}
