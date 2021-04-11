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
use crate::manifest::{Dependency, Module, Provider, Type, TypeRoot};
use crate::manifests::type_from_syn_type;
use crate::{environment, parsing};
use lazy_static::lazy_static;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::Token;
use syn::__private::TokenStream2;
use syn::parse_quote;
use syn::{Attribute, GenericArgument};

thread_local! {
    static MODULES :RefCell<HashMap<String, LocalModule>> = RefCell::new(HashMap::new());
}

lazy_static! {
    static ref MODULE_METADATA_KEYS: HashSet<String> = {
        let mut set = HashSet::<String>::new();
        set.insert("path".to_owned());
        set
    };
}

struct LocalModule {
    name: String,
    providers: Vec<Provider>,
    additional_path: Option<String>,
}

pub fn handle_module_attribute(
    attr: TokenStream,
    input: TokenStream,
) -> Result<TokenStream, TokenStream> {
    MODULES.with(|mm| {
        let span = input.span();
        let attributes = parsing::get_attribute_metadata(attr.clone())?;

        for key in attributes.keys() {
            if !MODULE_METADATA_KEYS.contains(key) {
                return spanned_compile_error(attr.span(), &format!("unknown key: {}", key));
            }
        }

        let module_path;
        let mut module_map = mm.borrow_mut();
        let mut item_impl: syn::ItemImpl =
            syn::parse2(input.clone()).map_spanned_compile_error(span, "impl expected")?;
        if let syn::Type::Path(path) = item_impl.self_ty.deref() {
            module_path = path.path.to_token_stream().to_string().replace(" ", "");
            if module_map.contains_key(&module_path) {
                return spanned_compile_error(span, "module was already declared");
            }
        } else {
            return spanned_compile_error(item_impl.span(), "path expected");
        }

        let mut module = LocalModule {
            name: module_path.to_owned(),
            additional_path: attributes.get("path").cloned(),
            providers: Vec::new(),
        };

        for i in 0..item_impl.items.len() {
            #[allow(unused_mut)] // required
            let mut item = item_impl.items.get_mut(i).unwrap();
            if let syn::ImplItem::Method(ref mut method) = item {
                let mut new_attrs: Vec<syn::Attribute> = Vec::new();
                for attr in &method.attrs {
                    if parsing::is_attribute(attr, "provides") {
                        handle_provides(attr, &mut module, &mut method.sig)?;
                    } else if parsing::is_attribute(attr, "binds") {
                        handle_binds(attr, &mut module, &mut method.sig, &mut method.block)?;
                        let allow_dead_code: Attribute = parse_quote! {#[allow(dead_code)]};
                        new_attrs.push(allow_dead_code);
                    } else {
                        new_attrs.push(attr.clone());
                    }
                }
                method.attrs = new_attrs;
            }
        }
        module_map.insert(module_path, module);

        Ok(quote! {#item_impl})
    })
}

fn handle_provides(
    attr: &syn::Attribute,
    module: &mut LocalModule,
    signature: &mut syn::Signature,
) -> Result<(), TokenStream2> {
    let mut provider = Provider::new();
    provider.name = signature.ident.to_string();
    if let syn::ReturnType::Type(ref _token, ref ty) = signature.output {
        provider.field_type = type_from_syn_type(ty.deref())?;
    } else {
        return spanned_compile_error(signature.span(), "return type expected");
    }
    for args in &signature.inputs {
        match args {
            syn::FnArg::Receiver(ref receiver) => {
                if receiver.reference.is_none() {
                    return spanned_compile_error(args.span(), "modules should not consume self");
                }
                provider.field_static = false;
            }
            syn::FnArg::Typed(ref type_) => {
                let mut dependency = Dependency::new();
                if let syn::Pat::Ident(ref ident) = type_.pat.deref() {
                    dependency.name = ident.ident.to_string()
                } else {
                    return spanned_compile_error(args.span(), "identifier expected");
                }
                dependency.field_type = type_from_syn_type(type_.ty.deref())?;
                provider.dependencies.push(dependency);
            }
        }
    }
    let provides_attr = parsing::get_parenthesized_attribute_metadata(attr.tokens.clone())?;
    let scopes = parsing::get_types(provides_attr.get("scope").map(Clone::clone))?;
    provider.field_type.scopes.extend(scopes);
    module.providers.push(provider);
    Ok(())
}

fn handle_binds(
    attr: &syn::Attribute,
    module: &mut LocalModule,
    signature: &mut syn::Signature,
    block: &mut syn::Block,
) -> Result<(), TokenStream2> {
    if !block.stmts.is_empty() {
        return spanned_compile_error(block.span(), "#[binds] methods must have empty body");
    }
    let body: syn::Stmt = syn::parse2(quote! { unimplemented!(); }).unwrap();
    block.stmts.push(body);

    let mut provider = Provider::new();
    provider.binds = true;
    provider.name = signature.ident.to_string();
    if let syn::ReturnType::Type(ref _token, ref mut ty) = signature.output {
        let return_type = type_from_syn_type(ty.deref())?;
        match return_type.path.borrow() {
            "lockjaw::MaybeScoped" => {}
            "MaybeScoped" => {}
            _ => {
                return spanned_compile_error(
                    signature.span(),
                    "#[binds] methods must return MaybeScoped<T>",
                )
            }
        }
        if let syn::Type::Path(ref mut type_path) = ty.deref_mut() {
            if let syn::PathArguments::AngleBracketed(ref mut angle_bracketed) =
                type_path.path.segments.last_mut().unwrap().arguments
            {
                if !has_lifetime(&angle_bracketed.args) {
                    let lifetime: GenericArgument = syn::parse2(quote! {'static}).unwrap();
                    angle_bracketed.args.push(lifetime);
                }
            }
        }
        provider.field_type = return_type.args[0].clone();
    } else {
        return spanned_compile_error(signature.span(), "return type expected");
    }
    if signature.inputs.len() != 1 {
        return spanned_compile_error(
            signature.span(),
            "binds method must only take the binding type as parameter",
        );
    }
    let args = signature.inputs.first().expect("missing binds arg");
    match args {
        syn::FnArg::Receiver(ref _receiver) => {
            return spanned_compile_error(
                args.span(),
                "binds method must only take the binding type as parameter",
            );
        }
        syn::FnArg::Typed(ref type_) => {
            let mut dependency = Dependency::new();
            if let syn::Pat::Ident(ref ident) = type_.pat.deref() {
                dependency.name = ident.ident.to_string();
            } else {
                return spanned_compile_error(args.span(), "identifier expected");
            }
            dependency.field_type = type_from_syn_type(type_.ty.deref())?;
            provider.dependencies.push(dependency);
        }
    }
    let provides_attr = parsing::get_parenthesized_attribute_metadata(attr.tokens.clone())?;
    let scopes = parsing::get_types(provides_attr.get("scope").map(Clone::clone))?;
    provider.field_type.scopes.extend(scopes);
    module.providers.push(provider);
    Ok(())
}

fn has_lifetime(args: &Punctuated<GenericArgument, Token![,]>) -> bool {
    for arg in args {
        if let GenericArgument::Lifetime(_) = arg {
            return true;
        }
    }
    false
}

pub fn generate_manifest(base_path: &str) -> Vec<Module> {
    MODULES.with(|m| {
        let mut modules = m.borrow_mut();
        let mut result = Vec::<Module>::new();
        for local_module in modules.values() {
            let mut module = Module::new();
            let mut type_ = Type::new();
            type_.field_crate = environment::current_crate();
            type_.root = TypeRoot::CRATE;
            let mut path = String::new();
            if !base_path.is_empty() {
                path.push_str(base_path);
                path.push_str("::");
            }
            if let Some(additional_path) = &local_module.additional_path {
                path.push_str(additional_path);
                path.push_str("::");
            }
            path.push_str(&local_module.name);

            type_.path = path;
            module.field_type = type_;
            module.providers.extend(local_module.providers.clone());
            result.push(module);
        }
        modules.clear();
        result
    })
}
