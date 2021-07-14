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

use std::collections::HashSet;
use std::ops::{Deref, DerefMut};

use lazy_static::lazy_static;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::Token;
use syn::{parse_quote, ImplItemMethod};
use syn::{Attribute, GenericArgument};

use crate::error::{spanned_compile_error, CompileError};
use crate::manifest::BindingType::{Binds, BindsOptionOf, Provides};
use crate::manifest::{
    with_manifest, Binding, BindingType, Dependency, Module, MultibindingMapKey, MultibindingType,
};
use crate::parsing;
use crate::parsing::{get_parenthesized_field_values, FieldValue};
use crate::prologue::prologue_check;
use crate::type_data::TypeData;
use std::convert::TryFrom;

lazy_static! {
    static ref MODULE_METADATA_KEYS: HashSet<String> = {
        let mut set = HashSet::<String>::new();
        set.insert("path".to_owned());
        set
    };
}

pub fn handle_module_attribute(
    attr: TokenStream,
    input: TokenStream,
) -> Result<TokenStream, TokenStream> {
    handle_module_attribute_internal(attr, input)
}

fn handle_module_attribute_internal(
    attr: TokenStream,
    input: TokenStream,
) -> Result<TokenStream, TokenStream> {
    let span = input.span();
    let attributes = parsing::get_attribute_field_values(attr.clone())?;

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
    } else {
        return spanned_compile_error(item_impl.span(), "path expected");
    }
    let mut bindings: Vec<Binding> = Vec::new();

    for i in 0..item_impl.items.len() {
        #[allow(unused_mut)] // required
        let mut item = item_impl.items.get_mut(i).unwrap();
        if let syn::ImplItem::Method(ref mut method) = item {
            bindings.push(parse_binding(method)?);
        }
    }

    let mut module = Module::new();
    module.type_data = TypeData::from_local(&module_path.to_owned(), item_impl.span())?;
    module.bindings.extend(bindings);
    with_manifest(|mut manifest| {
        for existing_module in &manifest.modules {
            if existing_module.type_data.eq(&module.type_data) {
                return spanned_compile_error(span, "module was already declared");
            }
        }
        Ok(manifest.modules.push(module))
    })?;

    let prologue_check = prologue_check(item_impl.span());
    Ok(quote! {
        #item_impl
        #prologue_check
    })
}

fn parse_binding(method: &mut ImplItemMethod) -> Result<Binding, TokenStream> {
    let mut option_binding: Option<Binding> = None;
    let mut multibinding = MultibindingType::None;
    let mut map_key = MultibindingMapKey::None;
    let mut new_attrs: Vec<syn::Attribute> = Vec::new();
    let mut qualifier: Option<Box<TypeData>> = None;
    for attr in &method.attrs {
        let attr_str = parsing::get_attribute(attr);
        match attr_str.as_str() {
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
                option_binding = Some(handle_binds(attr, &mut method.sig, &mut method.block)?);
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
            "qualified" => {
                qualifier = Some(Box::new(parsing::get_parenthesized_type(&attr.tokens)?));
            }
            "into_map" => {
                multibinding = MultibindingType::IntoMap;
                let fields = get_parenthesized_field_values(attr.tokens.clone())?;
                if let Some(field) = fields.get("string_key") {
                    if let FieldValue::StringLiteral(_, ref string) = field {
                        map_key = MultibindingMapKey::String(string.clone());
                    } else {
                        return spanned_compile_error(
                            attr.span(),
                            "string literal expected for string_key",
                        );
                    }
                } else if let Some(field) = fields.get("i32_key") {
                    if let FieldValue::IntLiteral(_, ref int) = field {
                        map_key = MultibindingMapKey::I32(
                            i32::try_from(*int)
                                .map_spanned_compile_error(attr.span(), "key overflows i32")?,
                        );
                    } else {
                        return spanned_compile_error(
                            attr.span(),
                            "i32 literal expected for i32_key",
                        );
                    }
                }
            }
            _ => {
                new_attrs.push(attr.clone());
            }
        }
    }
    method.attrs = new_attrs;
    if option_binding.is_none() {
        return spanned_compile_error(
            method.span(),
            "#[module] methods can only be annotated by #[provides]/#[binds]/#[binds_option_of]",
        );
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
            return spanned_compile_error(method.span(), "#[elements_into_set] must return Vec<T>");
        }
    }
    binding.multibinding_type = multibinding;
    binding.map_key = map_key;
    binding.type_data.qualifier = qualifier;
    Ok(binding)
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
    let provides_attr = parsing::get_parenthesized_field_values(attr.tokens.clone())?;
    let scopes = parsing::get_types(provides_attr.get("scope"), attr.span())?;
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
    let provides_attr = parsing::get_parenthesized_field_values(attr.tokens.clone())?;
    let scopes = parsing::get_types(provides_attr.get("scope"), attr.span())?;
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
    let provides_attr = parsing::get_parenthesized_field_values(attr.tokens.clone())?;
    let scopes = parsing::get_types(provides_attr.get("scope"), attr.span())?;
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
