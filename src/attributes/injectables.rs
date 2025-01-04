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

use std::collections::{HashMap, HashSet};

use crate::manifest_parser::Mod;
use crate::parsing::{
    get_attribute, get_attribute_field_values, get_parenthesized_field_values, get_type, get_types,
    has_attribute, is_attribute, FieldValue,
};
use crate::type_data::from_syn_type;
use anyhow::{bail, Context, Result};
use lazy_static::lazy_static;
use lockjaw_common::manifest::{Dependency, Injectable, Manifest, TypeRoot};
use lockjaw_common::type_data::TypeData;
use proc_macro2::TokenStream;
use syn::__private::quote::format_ident;
use syn::{FnArg, GenericArgument, ImplItem, ImplItemFn, Pat, PathArguments};

lazy_static! {
    static ref INJECTABLE_METADATA_KEYS: HashSet<String> = {
        let mut set = HashSet::<String>::new();
        set.insert("scope".to_owned());
        set.insert("container".to_owned());
        set
    };
}

lazy_static! {
    static ref FACTORY_METADATA_KEYS: HashSet<String> = {
        let mut set = HashSet::<String>::new();
        set.insert("implementing".to_owned());
        set.insert("visibility".to_owned());
        set
    };
}

#[derive(PartialEq)]
enum CtorType {
    Inject,
    Factory,
}
pub fn handle_injectable_attribute(
    attr: TokenStream,
    input: TokenStream,
    mod_: &Mod,
) -> Result<Manifest> {
    let mut item: syn::ItemImpl = syn::parse2(input).with_context(|| "impl block expected")?;

    let attributes = get_attribute_field_values(attr.clone())?;
    for key in attributes.keys() {
        if !INJECTABLE_METADATA_KEYS.contains(key) {
            bail!("unknown key: {}", key);
        }
    }
    let (ctor_type, ctor, fields) = get_ctor(&mut item.items)?;
    if ctor_type == CtorType::Factory {
        return handle_factory(item.self_ty.clone(), ctor.clone(), fields.clone(), mod_);
    }

    let mut dependencies = Vec::<Dependency>::new();
    for arg in ctor.sig.inputs.iter_mut() {
        if let FnArg::Receiver(_) = arg {
            bail!("self not allowed");
        }
        if let FnArg::Typed(ref mut type_) = arg {
            if let Pat::Ident(ref ident) = *type_.pat {
                let mut dependency = Dependency::new();
                dependency.type_data = from_syn_type(&type_.ty, mod_)?;
                let mut new_attrs = Vec::new();
                for attr in &type_.attrs {
                    match get_attribute(attr).as_str() {
                        "qualified" => {
                            dependency.type_data.qualifier = Some(Box::new(get_type(
                                &attr.meta.require_list().unwrap().tokens,
                                mod_,
                            )?))
                        }
                        _ => new_attrs.push(attr.clone()),
                    }
                }
                type_.attrs = Vec::new(); //new_attrs;
                dependency.name = ident.ident.to_string();
                dependencies.push(dependency);
            } else {
                bail!("identifier expected");
            }
        }
    }
    let type_name;
    let mut has_lifetime = false;
    if let syn::Type::Path(ref path) = *item.self_ty {
        let segments: Vec<String> = path
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect();
        type_name = segments.join("::");
        if let PathArguments::AngleBracketed(ref angle) =
            path.path.segments.last().as_ref().unwrap().arguments
        {
            for arg in &angle.args {
                if let GenericArgument::Lifetime(_) = arg {
                    has_lifetime = true;
                    break;
                }
            }
        }
    } else {
        bail!("path expected");
    }

    let mut injectable = Injectable::new();
    injectable.type_data = crate::type_data::from_local(&type_name, mod_)?;
    let scopes = get_types(attributes.get("scope"), mod_)?;

    injectable.container = get_container(mod_, &attributes, &scopes)?;
    injectable.type_data.scopes.extend(scopes);
    injectable.ctor_name = ctor.sig.ident.to_string();
    injectable.dependencies.extend(dependencies);

    let mut result = Manifest::new();

    if has_lifetime {
        result.lifetimed_types.insert(injectable.type_data.clone());
    }
    result.injectables.push(injectable);
    Ok(result)
}

fn get_ctor(
    items: &mut Vec<ImplItem>,
) -> Result<(CtorType, &mut ImplItemFn, HashMap<String, FieldValue>)> {
    let mut ctors = 0;
    for item in &mut *items {
        if let ImplItem::Fn(ref mut method) = item {
            if has_attribute(&method.attrs, "inject") || has_attribute(&method.attrs, "factory") {
                ctors += 1;
                if ctors == 2 {
                    bail!("only one method can be marked with #[inject]/#[factory]");
                }
            }
        }
    }
    if ctors == 0 {
        bail!("must have one method marked with #[inject]/#[factory]",);
    }
    for item in items {
        if let ImplItem::Fn(ref mut method) = item {
            if has_attribute(&method.attrs, "inject") {
                let index = method
                    .attrs
                    .iter()
                    .position(|a| is_attribute(a, "inject"))
                    .unwrap();
                let fields = get_parenthesized_field_values(&method.attrs[index].meta)?;
                method.attrs.remove(index);
                return Ok((CtorType::Inject, method, fields));
            }
            if has_attribute(&method.attrs, "factory") {
                let index = method
                    .attrs
                    .iter()
                    .position(|a| is_attribute(a, "factory"))
                    .unwrap();
                let fields = get_parenthesized_field_values(&method.attrs[index].meta)?;
                method.attrs.remove(index);
                return Ok((CtorType::Factory, method, fields));
            }
        }
    }
    panic!("should have ctor")
}

fn get_container(
    mod_: &Mod,
    attributes: &HashMap<String, FieldValue>,
    scopes: &Vec<TypeData>,
) -> Result<Option<TypeData>> {
    if attributes.contains_key("container") {
        if let FieldValue::Path(path) = attributes.get("container").unwrap() {
            if scopes.is_empty() {
                bail!("the 'container' metadata should only be used with an injectable that also has 'scope'",
                );
            }
            let container = crate::type_data::from_path(path, mod_)?;
            return Ok(Some(container));
        } else {
            bail!("path expected for 'container'");
        }
    }
    Ok(None)
}

fn handle_factory(
    mut self_ty: Box<syn::Type>,
    method: ImplItemFn,
    metadata: HashMap<String, FieldValue>,
    mod_: &Mod,
) -> Result<Manifest> {
    for (k, _) in &metadata {
        if !FACTORY_METADATA_KEYS.contains(k) {
            bail!("unknown key: {}", k);
        }
    }
    let mut dependencies = Vec::<Dependency>::new();
    for arg in method.sig.inputs.iter() {
        if let FnArg::Receiver(_) = arg {
            bail!("self not allowed");
        }
        if let FnArg::Typed(ref type_) = arg {
            if let Pat::Ident(ref ident) = *type_.pat {
                if !has_attribute(&type_.attrs, "runtime") {
                    let ty = &type_.ty;
                    let mut dependency = Dependency::new();
                    dependency.type_data = provider_type(&from_syn_type(ty, mod_)?);
                    dependency.name = ident.ident.to_string();

                    dependencies.push(dependency);
                }
            } else {
                bail!("identifier expected");
            }
        }
    }
    let mut factory_ty = self_ty.clone();
    if let syn::Type::Path(ref mut path) = self_ty.as_mut() {
        let last_segment = path.path.segments.last_mut().unwrap();
        if last_segment.arguments != PathArguments::None {
            last_segment.arguments = PathArguments::None;
        }

        let ident = format_ident!("{}Factory", path.path.segments.last().unwrap().ident);
        if let syn::Type::Path(ref mut factory_path) = factory_ty.as_mut() {
            let last_segment = factory_path.path.segments.last_mut().unwrap();
            last_segment.ident = ident;
            last_segment.arguments = PathArguments::None;
        }
    } else {
        bail!("path expected");
    }

    let mut injectable = Injectable::new();
    injectable.type_data = from_syn_type(&factory_ty, mod_)?;
    injectable.ctor_name = "lockjaw_new_factory".to_string();
    injectable.dependencies.extend(dependencies);

    let mut result = Manifest::new();

    result.lifetimed_types.insert(injectable.type_data.clone());

    result.injectables.push(injectable);

    Ok(result)
}

pub fn provider_type(type_: &TypeData) -> TypeData {
    let mut provider_type = TypeData::new();
    provider_type.root = TypeRoot::GLOBAL;
    provider_type.path = "lockjaw::Provider".to_string();
    provider_type.args.push(type_.clone());

    provider_type
}
