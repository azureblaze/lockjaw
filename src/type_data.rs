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
use anyhow::{bail, Context, Result};
use lazy_static::lazy_static;
use lockjaw_common::manifest::TypeRoot;
use lockjaw_common::type_data::TypeData;
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use syn::punctuated::Punctuated;
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
        m.insert("Cl".into(),"lockjaw::Cl".into() );
        m.insert("Provider".into(),"lockjaw::Provider".into() );
        m.insert("Lazy".into(),"lockjaw::Lazy".into() );
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

pub trait BuildScriptTypeData {
    /// Full path of the type in local from (use crate:: within the same crate), which can be
    /// converted to tokens.
    ///
    /// Modifiers like & are omitted
    fn syn_type(&self) -> syn::Type;
}
impl BuildScriptTypeData for TypeData {
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
}

pub fn from_local(identifier: &str, mod_: &Mod) -> Result<TypeData> {
    let mut result = TypeData::new();
    result.field_crate = mod_.crate_name.clone();
    result.root = TypeRoot::CRATE;
    result.path = mod_.resolve_declare_path(identifier)?;
    Ok(result)
}

pub fn from_syn_type(syn_type: &syn::Type, mod_: &Mod) -> Result<TypeData> {
    match syn_type {
        syn::Type::Path(ref type_path) => {
            return from_path(type_path.path.borrow(), mod_);
        }
        syn::Type::TraitObject(ref trait_object) => {
            let mut t: TypeData = from_type_param_bound(trait_object.bounds.borrow(), mod_)?;
            t.trait_object = true;
            return Ok(t);
        }
        syn::Type::ImplTrait(ref impl_trait) => {
            let mut t: TypeData = from_type_param_bound(impl_trait.bounds.borrow(), mod_)?;
            t.trait_object = true;
            return Ok(t);
        }
        syn::Type::Reference(ref reference) => {
            let mut t: TypeData = from_syn_type(reference.elem.deref(), mod_)?;
            t.field_ref = true;
            return Ok(t);
        }
        _ => bail!("unable to handle type {:?}", syn_type),
    }
}

pub fn from_type_param_bound(
    bounds: &Punctuated<TypeParamBound, syn::Token![+]>,
    mod_: &Mod,
) -> Result<TypeData> {
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
        bail!("one and only one trait expected");
    }
    let trait_ = traits.get(0).unwrap();
    from_path(&trait_.path, mod_)
}

pub fn from_path(syn_path: &syn::Path, mod_: &Mod) -> Result<TypeData> {
    let mut result = TypeData::new();
    let mut segment_iter = syn_path.segments.iter().peekable();
    if syn_path.leading_colon.is_some() {
        result.root = TypeRoot::GLOBAL;
        result
            .path
            .push_str(&segment_iter.next().unwrap().ident.to_string());
        result.path.push_str("::");
    } else if segment_iter
        .peek()
        .with_context(|| "empty segments")?
        .ident
        .to_string()
        .eq("crate")
    {
        segment_iter.next();
        result.root = TypeRoot::CRATE;
        result.field_crate = mod_.crate_name.clone();
    } else {
        let first = segment_iter
            .next()
            .with_context(|| "path segment expected")?;
        if segment_iter.peek().is_none() {
            if let Some(prelude) = PRELUDE_V1.get(&first.ident.to_string()) {
                result.path = prelude.clone();
                result.root = TypeRoot::GLOBAL;
                result.args.extend(get_args(first, mod_)?);
                return Ok(result);
            }
            if PRIMITIVES.contains(&first.ident.to_string()) {
                result.path = first.ident.to_string();
                result.root = TypeRoot::PRIMITIVE;
                result.args.extend(get_args(first, mod_)?);
                return Ok(result);
            }
        }
        result = mod_.resolve_path(
            &first.ident.to_string())
            .with_context(|| "lockjaw is unable to resolve the type, consider using fully qualified path (start with \"::\" or \"crate::\")",
            )?;
        if segment_iter.peek().is_some() {
            result.path.push_str("::");
        } else {
            result.args.extend(get_args(first, mod_)?);
            return Ok(result);
        }
    }
    if segment_iter.peek().is_some() {
        while let Some(segment) = segment_iter.next() {
            result.path.push_str(&segment.ident.to_string());
            if let Some(_) = segment_iter.peek() {
                result.path.push_str("::");
                if !segment.arguments.is_empty() {
                    bail!("arguments only supported in the last segment of the path",);
                }
            } else {
                result.args.extend(get_args(&segment, mod_)?);
            }
        }
    }
    Ok(result)
}

fn get_args(segment: &syn::PathSegment, mod_: &Mod) -> Result<Vec<TypeData>> {
    let mut result = Vec::<TypeData>::new();
    if let syn::PathArguments::AngleBracketed(ref angle) = segment.arguments {
        for generic_arg in &angle.args {
            match generic_arg {
                syn::GenericArgument::Type(ref type_) => result.push(from_syn_type(type_, mod_)?),
                syn::GenericArgument::Lifetime(ref _lifetime) => {
                    // Do nothing
                }
                _ => {
                    bail!("unable to handle generic argument")
                }
            }
        }
    }
    Ok(result)
}
