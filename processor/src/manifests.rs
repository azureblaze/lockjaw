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

use crate::environment;
use lazy_static::lazy_static;
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::ops::{AddAssign, Deref};

use crate::error::{spanned_compile_error, CompileError};
use crate::manifest::{Type, TypeRoot};
use proc_macro2::TokenStream;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{TraitBound, TypeParamBound};

lazy_static! {
    /// auto used types that does not need fully qualified paths.
    static ref PRELUDE_V1: HashMap<String, String> = {
        let mut m = HashMap::<String, String>::new();
        m.insert("Box".into(), "std::boxed::Box".into());
        m.insert("Option".into(), "std::option::Option".into());
        m.insert("Result".into(), "std::result::Result".into());
        m.insert("String".into(), "std::string::String".into());
        m.insert("Vec".into(), "std::vec::Vec".into());
        m.insert("MaybeScoped".into(),"lockjaw::MaybeScoped".into() );
        m
    };
}

lazy_static! {
    /// primitive data types with no path
    static ref PRIMITIVES: HashSet<String> = {
        let mut m = HashSet::<String>::new();
        m.insert("i8".to_owned());
        m.insert("u8".to_owned());
        m.insert("i16".to_owned());
        m.insert("u16".to_owned());
        m.insert("i32".to_owned());
        m.insert("u32".to_owned());
        m.insert("i64".to_owned());
        m.insert("u64".to_owned());
        m.insert("i128".to_owned());
        m.insert("u128".to_owned());
        m.insert("isize".to_owned());
        m.insert("usize".to_owned());
        m.insert("f32".to_owned());
        m.insert("f64".to_owned());
        m.insert("bool".to_owned());
        m.insert("char".to_owned());
        m
    };
}

pub fn type_from_syn_type(syn_type: &syn::Type) -> Result<Type, TokenStream> {
    match syn_type {
        syn::Type::Path(ref type_path) => {
            return type_from_path(type_path.path.borrow());
        }
        syn::Type::TraitObject(ref trait_object) => {
            let mut t: Type = type_from_type_param_bound(trait_object.bounds.borrow())?;
            t.trait_object = true;
            return Ok(t);
        }
        syn::Type::ImplTrait(ref impl_trait) => {
            let mut t: Type = type_from_type_param_bound(impl_trait.bounds.borrow())?;
            t.trait_object = true;
            return Ok(t);
        }
        syn::Type::Reference(ref reference) => {
            let mut t: Type = type_from_syn_type(reference.elem.deref())?;
            t.field_ref = true;
            return Ok(t);
        }
        _ => {
            return spanned_compile_error(
                syn_type.span(),
                &format!("unable to handle type {:?}", syn_type),
            );
        }
    }
}

pub fn type_from_type_param_bound(
    bounds: &Punctuated<TypeParamBound, syn::Token![+]>,
) -> Result<Type, TokenStream> {
    let traits = bounds
        .iter()
        .filter_map(|bound| {
            if let syn::TypeParamBound::Trait(ref trait_) = bound {
                return Some(trait_);
            }
            return None;
        })
        .collect::<Vec<&TraitBound>>();
    if traits.len() != 1 {
        return spanned_compile_error(bounds.span(), "one and only one trait expected");
    }
    let trait_ = traits.get(0).unwrap();
    return type_from_path(&trait_.path);
}

pub fn type_from_path(syn_path: &syn::Path) -> Result<Type, TokenStream> {
    let mut result = Type::new();
    let mut segment_iter = syn_path.segments.iter().peekable();
    if syn_path.leading_colon.is_some() {
        result.root = TypeRoot::GLOBAL;
    } else if segment_iter
        .peek()
        .map_spanned_compile_error(syn_path.span(), "empty segments")?
        .ident
        .to_string()
        .eq("crate")
    {
        segment_iter.next();
        result.root = TypeRoot::CRATE;
        result.field_crate = environment::current_crate()
    } else {
        let first = segment_iter
            .next()
            .map_spanned_compile_error(syn_path.span(), "path segment expected")?;
        if segment_iter.next().is_none() {
            if let Some(prelude) = PRELUDE_V1.get(&first.ident.to_string()) {
                result.path = prelude.clone();
                result.root = TypeRoot::GLOBAL;
                result.args.extend(get_args(first)?);
                return Ok(result);
            }
            if PRIMITIVES.contains(&first.ident.to_string()) {
                result.path = first.ident.to_string();
                result.root = TypeRoot::PRIMITIVE;
                result.args.extend(get_args(first)?);
                return Ok(result);
            }
        }
        return spanned_compile_error(
            syn_path.span(),
            "types must be fully qualified. it should either start with \"::\" or \"crate::\"",
        );
    }
    let mut path = String::new();
    while let Some(segment) = segment_iter.next() {
        path.add_assign(&segment.ident.to_string());
        if let Some(_) = segment_iter.peek() {
            path.add_assign("::");
            if !segment.arguments.is_empty() {
                return spanned_compile_error(
                    segment.span(),
                    "arguments only supported in the last segment of the path",
                );
            }
        } else {
            result.args.extend(get_args(&segment)?);
        }
    }
    result.path = path;
    Ok(result)
}

fn get_args(segment: &syn::PathSegment) -> Result<Vec<Type>, TokenStream> {
    let mut result = Vec::<Type>::new();
    if let syn::PathArguments::AngleBracketed(ref angle) = segment.arguments {
        for generic_arg in &angle.args {
            match generic_arg {
                syn::GenericArgument::Type(ref type_) => result.push(type_from_syn_type(type_)?),
                syn::GenericArgument::Lifetime(ref _lifetime) => {
                    // Do nothing
                }
                _ => {
                    return spanned_compile_error(
                        segment.span(),
                        "unable to handle generic argument",
                    )
                }
            }
        }
    }
    Ok(result)
}
