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

use std::cell::RefCell;
use std::collections::HashSet;

use lazy_static::lazy_static;
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{FnArg, ImplItem, ImplItemMethod, Pat};

use crate::error::{spanned_compile_error, CompileError};
use crate::manifest::{Dependency, Injectable};
use crate::parsing;
use crate::type_data::TypeData;

struct LocalInjectable {
    identifier: String,
    additional_path: Option<String>,
    scopes: Vec<TypeData>,
    ctor_name: String,
    dependencies: Vec<Dependency>,
}

thread_local! {
    static INJECTABLES :RefCell<Vec<LocalInjectable>> = RefCell::new(Vec::new());
}

lazy_static! {
    static ref INJECTABLE_METADATA_KEYS: HashSet<String> = {
        let mut set = HashSet::<String>::new();
        set.insert("scope".to_owned());
        set.insert("path".to_owned());
        set
    };
}

pub fn handle_injectable_attribute(
    attr: TokenStream,
    input: TokenStream,
) -> Result<TokenStream, TokenStream> {
    let span = input.span();
    let mut item: syn::ItemImpl =
        syn::parse2(input).map_spanned_compile_error(span, "impl block expected")?;

    let attributes = parsing::get_attribute_metadata(attr.clone())?;
    for key in attributes.keys() {
        if !INJECTABLE_METADATA_KEYS.contains(key) {
            return spanned_compile_error(attr.span(), &format!("unknown key: {}", key));
        }
    }

    let ctor = get_ctor(item.span(), &mut item.items)?;
    let mut dependencies = Vec::<Dependency>::new();
    for arg in ctor.sig.inputs.iter_mut() {
        if let FnArg::Receiver(ref receiver) = arg {
            return spanned_compile_error(receiver.span(), &format!("self not allowed"));
        }
        if let FnArg::Typed(ref mut type_) = arg {
            if let Pat::Ident(ref ident) = *type_.pat {
                let mut dependency = Dependency::new();
                dependency.type_data = TypeData::from_syn_type(&type_.ty)?;
                let mut new_attrs = Vec::new();
                for attr in &type_.attrs {
                    match parsing::get_attribute(attr).as_str() {
                        "qualified" => {
                            dependency.type_data.qualifier =
                                Some(Box::new(parsing::get_parenthesized_type(&attr.tokens)?))
                        }
                        _ => new_attrs.push(attr.clone()),
                    }
                }
                type_.attrs = Vec::new(); //new_attrs;
                dependency.name = ident.ident.to_string();
                dependencies.push(dependency);
            } else {
                return spanned_compile_error(type_.span(), &format!("identifier expected"));
            }
        }
    }
    let type_name;
    if let syn::Type::Path(ref path) = *item.self_ty {
        let segments: Vec<String> = path
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect();
        type_name = segments.join("::");
    } else {
        return spanned_compile_error(item.self_ty.span(), &format!("path expected"));
    }

    let injectable = LocalInjectable {
        identifier: type_name,
        additional_path: attributes.get("path").cloned(),
        scopes: parsing::get_types(attributes.get("scope").map(Clone::clone))?,
        ctor_name: ctor.sig.ident.to_string(),
        dependencies,
    };
    INJECTABLES.with(|injectables| {
        injectables.borrow_mut().push(injectable);
    });

    Ok(item.to_token_stream())
}

fn get_ctor(span: Span, items: &mut Vec<ImplItem>) -> Result<&mut ImplItemMethod, TokenStream> {
    let mut ctors = 0;
    for item in &mut *items {
        if let ImplItem::Method(ref mut method) = item {
            if parsing::has_attribute(&method.attrs, "inject") {
                ctors += 1;
                if ctors == 2 {
                    return spanned_compile_error(
                        item.span(),
                        "only one method can be marked with #[inject]",
                    );
                }
            }
        }
    }
    if ctors == 0 {
        return spanned_compile_error(span, "must have one method marked with #[inject]");
    }
    for item in items {
        if let ImplItem::Method(ref mut method) = item {
            if parsing::has_attribute(&method.attrs, "inject") {
                let index = method
                    .attrs
                    .iter()
                    .position(|a| parsing::is_attribute(a, "inject"))
                    .unwrap();
                method.attrs.remove(index);
                return Ok(method);
            }
        }
    }
    panic!("should have ctor")
}

pub fn generate_manifest(base_path: &str) -> Vec<Injectable> {
    INJECTABLES.with(|injectables| {
        let mut result = Vec::new();
        for local_injectable in injectables.borrow().iter() {
            let mut injectable = Injectable::new();
            injectable.type_data = TypeData::from_local(
                base_path,
                &local_injectable.additional_path,
                &local_injectable.identifier,
            );
            injectable
                .type_data
                .scopes
                .extend(local_injectable.scopes.clone());
            injectable.ctor_name = local_injectable.ctor_name.clone();
            injectable
                .dependencies
                .extend(local_injectable.dependencies.clone());

            result.push(injectable);
        }
        injectables.borrow_mut().clear();
        result
    })
}
