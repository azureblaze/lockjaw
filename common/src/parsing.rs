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

use crate::manifest_parser::Mod;
use crate::type_data::TypeData;
use anyhow::{bail, Context, Result};
use proc_macro2::TokenStream;
use std::collections::HashMap;
use syn::parse::Parser;
#[allow(unused_imports)] // somehow rust think this is unused.
use syn::spanned::Spanned;
use syn::{Attribute, Meta};

pub fn is_attribute(syn_attr: &Attribute, attr: &str) -> bool {
    get_attribute(syn_attr).eq(attr)
}

pub fn get_attribute(syn_attr: &Attribute) -> String {
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
#[allow(dead_code)]
pub fn find_attribute<'a, 'b>(attrs: &'a Vec<Attribute>, attr: &'b str) -> Option<&'a Attribute> {
    attrs.iter().find(|a| is_attribute(a, attr))
}

pub fn has_attribute(attrs: &Vec<Attribute>, attr: &str) -> bool {
    attrs.iter().find(|a| is_attribute(a, attr)).is_some()
}

pub fn get_parenthesized_field_values(meta: &Meta) -> Result<HashMap<String, FieldValue>> {
    match meta {
        Meta::Path(_) => Ok(HashMap::new()),
        Meta::List(list) => get_attribute_field_values(list.tokens.clone()),
        Meta::NameValue(_) => {
            bail!("list expected")
        }
    }
}
#[allow(dead_code)]
pub fn get_path(attr: &TokenStream) -> Result<syn::Path> {
    if attr.is_empty() {
        bail!("path expected");
    }
    syn::parse2(attr.clone()).with_context(|| "path expected")
}

pub fn get_type(attr: &TokenStream, mod_: &Mod) -> Result<TypeData> {
    if attr.is_empty() {
        bail!("path expected");
    }
    crate::type_data::from_path(
        &syn::parse2(attr.clone()).with_context(|| "path expected")?,
        mod_,
    )
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum FieldValue {
    StringLiteral(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    BoolLiteral(bool),
    Path(syn::Path),
    Array(Vec<FieldValue>),
    FieldValues(HashMap<String, FieldValue>),
}

impl FieldValue {
    #[allow(dead_code)]
    pub fn get_paths(&self) -> Result<Vec<syn::Path>> {
        match self {
            FieldValue::Path(ref path) => Ok(vec![path.clone()]),
            FieldValue::Array(ref array) => array
                .iter()
                .map(|f| {
                    if let FieldValue::Path(ref path) = f {
                        Ok(path.clone())
                    } else {
                        bail!("path expected")
                    }
                })
                .collect(),
            _ => bail!("path expected"),
        }
    }
    #[allow(dead_code)]
    pub fn get_types(&self, mod_: &Mod) -> Result<Vec<TypeData>> {
        let mut result = Vec::new();
        for path in self.get_paths()? {
            result.push(crate::type_data::from_path(&path, mod_)?)
        }
        Ok(result)
    }
}

/// Converts #[attr(key1 : "value1", key2 : value2)] to key-value map.
pub fn get_attribute_field_values(attr: TokenStream) -> Result<HashMap<String, FieldValue>> {
    let parser = syn::punctuated::Punctuated::<syn::FieldValue, syn::Token![,]>::parse_terminated;
    if attr.is_empty() {
        return Ok(HashMap::new());
    }

    let field_values = parser
        .parse2(attr.clone())
        .with_context(|| "FieldValue (key: value, ...) expected")?;

    parse_punctuated_field_values(&field_values)
}

fn parse_punctuated_field_values(
    field_values: &syn::punctuated::Punctuated<syn::FieldValue, syn::Token![,]>,
) -> Result<HashMap<String, FieldValue>> {
    let mut result = HashMap::new();
    for field in field_values.iter() {
        if let syn::Member::Named(ref name) = field.member {
            result.insert(name.to_string(), parse_field_value(&field.expr)?);
        } else {
            bail!("field should have named member");
        }
    }
    Ok(result)
}

fn parse_field_value(expr: &syn::Expr) -> Result<FieldValue> {
    match expr {
        syn::Expr::Lit(ref lit) => match lit.lit {
            syn::Lit::Str(ref str_) => Ok(FieldValue::StringLiteral(str_.value())),
            syn::Lit::Bool(ref bool_) => Ok(FieldValue::BoolLiteral(bool_.value())),
            syn::Lit::Int(ref int) => Ok(FieldValue::IntLiteral(
                int.base10_parse::<i64>()
                    .with_context(|| "unable to parse integer to i64")?,
            )),
            syn::Lit::Float(ref float) => Ok(FieldValue::FloatLiteral(
                float
                    .base10_parse::<f64>()
                    .with_context(|| "unable to parse integer to f64")?,
            )),
            _ => bail!("unable to handle literal value {:?}", lit),
        },
        syn::Expr::Path(ref path) => Ok(FieldValue::Path(path.path.clone())),
        syn::Expr::Array(ref array) => {
            let mut values: Vec<FieldValue> = Vec::new();
            for expr in &array.elems {
                values.push(parse_field_value(expr)?);
            }
            Ok(FieldValue::Array(values))
        }
        syn::Expr::Struct(ref struct_) => Ok(FieldValue::FieldValues(
            parse_punctuated_field_values(&struct_.fields)?,
        )),
        _ => bail!("invalid field value {:?}", expr),
    }
}

/// Parses "foo::Bar, foo::Baz" to a list of types.
pub fn get_types(types: Option<&FieldValue>, mod_: &Mod) -> Result<Vec<TypeData>> {
    if types.is_none() {
        return Ok(Vec::new());
    }
    match types.unwrap() {
        FieldValue::Path(ref path) => Ok(vec![crate::type_data::from_path(path, mod_)?]),
        FieldValue::Array(ref paths) => {
            let mut result = Vec::new();
            for field in paths {
                if let FieldValue::Path(ref path) = field {
                    result.push(crate::type_data::from_path(path, mod_)?);
                } else {
                    bail!("field in array is not a path");
                }
            }
            Ok(result)
        }
        _ => bail!("path or [path, ...] expected"),
    }
}
