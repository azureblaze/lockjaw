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

use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};

use lazy_static::lazy_static;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse_quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::Token;
use syn::{Attribute, GenericArgument};

use crate::error::{spanned_compile_error, CompileError};
use crate::manifest::BindingType::{Binds, BindsOptionOf, Provides};
use crate::manifest::{Binding, BindingType, Dependency, Module, MultibindingType, TypeRoot};
use crate::type_data::TypeData;
use crate::{environment, parsing};

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
    bindings: Vec<Binding>,
    additional_path: Option<String>,
}

pub fn handle_module_attribute(
    attr: TokenStream,
    input: TokenStream,
) -> Result<TokenStream, TokenStream> {
    MODULES.with(|mm| handle_module_attribute_internal(attr, input, mm.borrow_mut()))
}

fn handle_module_attribute_internal(
    attr: TokenStream,
    input: TokenStream,
    mut module_map: RefMut<HashMap<String, LocalModule>>,
) -> Result<TokenStream, TokenStream> {
    let span = input.span();
    let attributes = parsing::get_attribute_metadata(attr.clone())?;

    for key in attributes.keys() {
        if !MODULE_METADATA_KEYS.contains(key) {
            return spanned_compile_error(attr.span(), &format!("unknown key: {}", key));
        }
    }

    let module_path;
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
        bindings: Vec::new(),
    };

    for i in 0..item_impl.items.len() {
        #[allow(unused_mut)] // required
        let mut item = item_impl.items.get_mut(i).unwrap();
        if let syn::ImplItem::Method(ref mut method) = item {
            let mut option_binding: Option<Binding> = None;
            let mut multibinding = MultibindingType::None;
            let mut new_attrs: Vec<syn::Attribute> = Vec::new();
            for attr in &method.attrs {
                match parsing::get_attribute(attr).as_str() {
                    "provides" => {
                        if option_binding.is_some() {
                            return spanned_compile_error(attr.span(), "#[module] methods can only be annotated by one of #[provides]/#[binds]/#[binds_option_of]");
                        }
                        option_binding = Some(handle_provides(attr, &mut method.sig)?);
                    }
                    "binds" => {
                        if option_binding.is_some() {
                            return spanned_compile_error(attr.span(), "#[module] methods can only be annotated by one of #[provides]/#[binds]/#[binds_option_of]");
                        }
                        option_binding =
                            Some(handle_binds(attr, &mut method.sig, &mut method.block)?);
                        let allow_dead_code: Attribute = parse_quote! {#[allow(dead_code)]};
                        new_attrs.push(allow_dead_code);
                        let allow_unused: Attribute = parse_quote! {#[allow(unused)]};
                        new_attrs.push(allow_unused);
                    }
                    "binds_option_of" => {
                        if option_binding.is_some() {
                            return spanned_compile_error(attr.span(), "#[module] methods can only be annotated by one of #[provides]/#[binds]/#[binds_option_of]");
                        }
                        option_binding = Some(handle_binds_option_of(
                            attr,
                            &mut method.sig,
                            &mut method.block,
                        )?);
                        let allow_dead_code: Attribute = parse_quote! {#[allow(dead_code)]};
                        new_attrs.push(allow_dead_code);
                    }
                    "into_vec" => {
                        multibinding = MultibindingType::IntoVec;
                    }
                    "elements_into_vec" => {
                        multibinding = MultibindingType::ElementsIntoVec;
                    }
                    _ => {
                        new_attrs.push(attr.clone());
                    }
                }
            }
            method.attrs = new_attrs;
            if option_binding.is_none() {
                return spanned_compile_error(attr.span(), "#[module] methods can only be annotated by #[provides]/#[binds]/#[binds_option_of]");
            }
            let mut binding = option_binding.unwrap();
            if binding.binding_type == BindingType::Binds {
                if multibinding == MultibindingType::ElementsIntoVec {
                    return spanned_compile_error(
                        method.span(),
                        "#[elements_into_set] cannot be used on #[binds]",
                    );
                }
            }

            if multibinding == MultibindingType::ElementsIntoVec {
                if binding.type_data.path.ne("std::vec::Vec") {
                    return spanned_compile_error(
                        method.span(),
                        "#[elements_into_set] must return Vec<T>",
                    );
                }
            }
            binding.multibinding_type = multibinding;
            module.bindings.push(binding);
        }
    }
    module_map.insert(module_path, module);

    Ok(quote! {#item_impl})
}

fn handle_provides(
    attr: &syn::Attribute,
    signature: &mut syn::Signature,
) -> Result<Binding, TokenStream> {
    let mut provides = Binding::new(Provides);
    provides.name = signature.ident.to_string();
    if let syn::ReturnType::Type(ref _token, ref ty) = signature.output {
        provides.type_data = TypeData::from_syn_type(ty.deref())?;
    } else {
        return spanned_compile_error(signature.span(), "return type expected");
    }
    for args in &signature.inputs {
        match args {
            syn::FnArg::Receiver(ref receiver) => {
                if receiver.reference.is_none() {
                    return spanned_compile_error(args.span(), "modules should not consume self");
                }
                provides.field_static = false;
            }
            syn::FnArg::Typed(ref type_) => {
                let mut dependency = Dependency::new();
                if let syn::Pat::Ident(ref ident) = type_.pat.deref() {
                    dependency.name = ident.ident.to_string()
                } else {
                    return spanned_compile_error(args.span(), "identifier expected");
                }
                dependency.type_data = TypeData::from_syn_type(type_.ty.deref())?;
                provides.dependencies.push(dependency);
            }
        }
    }
    let provides_attr = parsing::get_parenthesized_attribute_metadata(attr.tokens.clone())?;
    let scopes = parsing::get_types(provides_attr.get("scope").map(Clone::clone))?;
    provides.type_data.scopes.extend(scopes);
    Ok(provides)
}

fn handle_binds(
    attr: &syn::Attribute,
    signature: &mut syn::Signature,
    block: &mut syn::Block,
) -> Result<Binding, TokenStream> {
    if !block.stmts.is_empty() {
        return spanned_compile_error(block.span(), "#[binds] methods must have empty body");
    }
    let body: syn::Stmt = syn::parse2(quote! { unimplemented!(); }).unwrap();
    block.stmts.push(body);

    let mut binds = Binding::new(Binds);
    binds.name = signature.ident.to_string();
    if let syn::ReturnType::Type(ref _token, ref mut ty) = signature.output {
        let return_type = TypeData::from_syn_type(ty.deref())?;
        match return_type.path.as_str() {
            "lockjaw::ComponentLifetime" => {}
            "ComponentLifetime" => {}
            _ => {
                return spanned_compile_error(
                    signature.span(),
                    "#[binds] methods must return ComponentLifetime<T>",
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
        binds.type_data = return_type.args[0].clone();
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
            dependency.type_data = TypeData::from_syn_type(type_.ty.deref())?;
            binds.dependencies.push(dependency);
        }
    }
    let provides_attr = parsing::get_parenthesized_attribute_metadata(attr.tokens.clone())?;
    let scopes = parsing::get_types(provides_attr.get("scope").map(Clone::clone))?;
    binds.type_data.scopes.extend(scopes);
    Ok(binds)
}

fn handle_binds_option_of(
    attr: &syn::Attribute,
    signature: &mut syn::Signature,
    block: &mut syn::Block,
) -> Result<Binding, TokenStream> {
    if !block.stmts.is_empty() {
        return spanned_compile_error(
            block.span(),
            "#[binds_option_of] methods must have empty body",
        );
    }
    let body: syn::Stmt = syn::parse2(quote! { unimplemented!(); }).unwrap();
    block.stmts.push(body);

    let mut binds_option_of = Binding::new(BindsOptionOf);
    if let syn::ReturnType::Type(ref _token, ref mut ty) = signature.output {
        let return_type = TypeData::from_syn_type(ty.deref())?;
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
        binds_option_of.type_data = return_type;
    } else {
        return spanned_compile_error(signature.span(), "return type expected");
    }
    if signature.inputs.len() != 0 {
        return spanned_compile_error(
            signature.span(),
            "binds_option_of method must only take no parameter",
        );
    }
    let provides_attr = parsing::get_parenthesized_attribute_metadata(attr.tokens.clone())?;
    let scopes = parsing::get_types(provides_attr.get("scope").map(Clone::clone))?;
    binds_option_of.type_data.scopes.extend(scopes);
    Ok(binds_option_of)
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
            let mut type_ = TypeData::new();
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
            module.type_data = type_;
            module.bindings.extend(local_module.bindings.clone());
            result.push(module);
        }
        modules.clear();
        result
    })
}
