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
use syn::parse_quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Attribute, GenericArgument};
use syn::{ImplItemFn, Token};

use crate::error::{spanned_compile_error, CompileError};
use crate::manifest::with_manifest;
use crate::parsing;
use crate::parsing::{get_parenthesized_field_values, FieldValue};
use crate::prologue::prologue_check;
use crate::type_validator::TypeValidator;
use lockjaw_common::manifest::BindingType::{Binds, BindsOptionOf, Multibinds, Provides};
use lockjaw_common::manifest::{
    Binding, BindingType, Dependency, Module, MultibindingMapKey, MultibindingType,
};
use lockjaw_common::type_data::TypeData;
use std::convert::TryFrom;
use std::iter::FromIterator;

lazy_static! {
    static ref MODULE_METADATA_KEYS: HashSet<String> = {
        let mut set = HashSet::<String>::new();
        set.insert("subcomponents".to_owned());
        set.insert("install_in".to_owned());
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
    let mut type_validator = TypeValidator::new();
    for i in 0..item_impl.items.len() {
        #[allow(unused_mut)] // required
        let mut item = item_impl.items.get_mut(i).unwrap();
        if let syn::ImplItem::Fn(ref mut method) = item {
            bindings.push(parse_binding(method, &mut type_validator)?);
        }
    }

    let mut module = Module::new();
    module.type_data = crate::type_data::from_local(&module_path.to_owned(), item_impl.span())?;
    module.bindings.extend(bindings);
    if let Some(subcomponents) = attributes.get("subcomponents") {
        let types = subcomponents.get_types()?;
        for type_ in &types {
            type_validator.add_dyn_type(type_, attr.span());
        }
        let paths = subcomponents.get_paths()?;
        for (path, span) in &paths {
            type_validator.add_dyn_path(path, span.clone());
        }
        module.subcomponents = HashSet::from_iter(types);
    }
    if let Some(install_in) = attributes.get("install_in") {
        let types = install_in.get_types()?;
        for type_ in &types {
            type_validator.add_dyn_type(type_, attr.span());
        }
        let paths = install_in.get_paths()?;
        for (path, span) in &paths {
            type_validator.add_dyn_path(path, span.clone());
        }
        module.install_in = HashSet::from_iter(types);
    }

    let validate_type = type_validator.validate(module.type_data.identifier_string());
    with_manifest(|mut manifest| {
        for existing_module in &manifest.modules {
            if existing_module.type_data.eq(&module.type_data) {
                return spanned_compile_error(span, "module was already declared");
            }
        }
        Ok(manifest.modules.push(module))
    })?;

    let prologue_check = prologue_check(item_impl.span());
    let result = quote! {
        #item_impl
        #validate_type
        #prologue_check
    };
    Ok(result)
}

fn parse_binding(
    method: &mut ImplItemFn,
    type_validator: &mut TypeValidator,
) -> Result<Binding, TokenStream> {
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
                    return spanned_compile_error(attr.span(), "#[module] methods can only be annotated by one of #[provides]/#[binds]/#[binds_option_of]/#[multibinds]");
                }
                option_binding = Some(handle_provides(attr, &mut method.sig, type_validator)?);
            }
            "binds" => {
                if option_binding.is_some() {
                    return spanned_compile_error(attr.span(), "#[module] methods can only be annotated by one of #[provides]/#[binds]/#[binds_option_of]/#[multibinds]");
                }
                option_binding = Some(handle_binds(
                    attr,
                    &mut method.sig,
                    &mut method.block,
                    type_validator,
                )?);
                let allow_dead_code: Attribute = parse_quote! {#[allow(dead_code)]};
                new_attrs.push(allow_dead_code);
                let allow_unused: Attribute = parse_quote! {#[allow(unused)]};
                new_attrs.push(allow_unused);
            }
            "binds_option_of" => {
                if option_binding.is_some() {
                    return spanned_compile_error(attr.span(), "#[module] methods can only be annotated by one of #[provides]/#[binds]/#[binds_option_of]/#[multibinds]");
                }
                option_binding = Some(handle_binds_option_of(&mut method.sig, &mut method.block)?);
                let allow_dead_code: Attribute = parse_quote! {#[allow(dead_code)]};
                new_attrs.push(allow_dead_code);
            }
            "multibinds" => {
                if option_binding.is_some() {
                    return spanned_compile_error(attr.span(), "#[module] methods can only be annotated by one of #[provides]/#[binds]/#[binds_option_of]/#[multibinds]");
                }
                option_binding = Some(handle_multibinds(&mut method.sig, &mut method.block)?);
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
                qualifier = Some(Box::new(parsing::get_type(
                    &attr.meta.require_list().unwrap().tokens,
                )?));
            }
            "into_map" => {
                multibinding = MultibindingType::IntoMap;
                let fields = get_parenthesized_field_values(&attr.meta)?;
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
                } else if let Some(field) = fields.get("enum_key") {
                    if let FieldValue::Path(span, ref path) = field {
                        let value_type = crate::type_data::from_path_with_span(path, span.clone())?;
                        let mut enum_type = value_type.clone();
                        enum_type.path.truncate(
                            enum_type.path.rfind("::").map_spanned_compile_error(
                                span.clone(),
                                "enum value should have at least one segment",
                            )?,
                        );
                        map_key = MultibindingMapKey::Enum(enum_type, value_type);
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
    type_validator: &mut TypeValidator,
) -> Result<Binding, TokenStream> {
    let mut provides = Binding::new(Provides);
    provides.name = signature.ident.to_string();
    if let syn::ReturnType::Type(ref _token, ref ty) = signature.output {
        provides.type_data = crate::type_data::from_syn_type(ty.deref())?;
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
                dependency.type_data = crate::type_data::from_syn_type(type_.ty.deref())?;
                provides.dependencies.push(dependency);
            }
        }
    }
    let provides_attr = parsing::get_parenthesized_field_values(&attr.meta)?;
    if let Some(scope) = provides_attr.get("scope") {
        let scopes = parsing::get_types(Some(scope), attr.span())?;
        for scope in &scopes {
            type_validator.add_dyn_type(scope, attr.span());
        }
        for (path, span) in scope.get_paths()? {
            type_validator.add_dyn_path(&path, span);
        }
        provides.type_data.scopes.extend(scopes);
    }
    Ok(provides)
}

fn handle_binds(
    attr: &syn::Attribute,
    signature: &mut syn::Signature,
    block: &mut syn::Block,
    type_validator: &mut TypeValidator,
) -> Result<Binding, TokenStream> {
    if !block.stmts.is_empty() {
        return spanned_compile_error(block.span(), "#[binds] methods must have empty body");
    }
    let body: syn::Stmt = syn::parse2(quote! { unimplemented!(); }).unwrap();
    block.stmts.push(body);

    let mut binds = Binding::new(Binds);
    binds.name = signature.ident.to_string();
    if let syn::ReturnType::Type(ref _token, ref mut ty) = signature.output {
        let return_type = crate::type_data::from_syn_type(ty.deref())?;
        match return_type.path.as_str() {
            "lockjaw::Cl" => {}
            "Cl" => {}
            _ => {
                return spanned_compile_error(
                    signature.span(),
                    "#[binds] methods must return Cl<T>",
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
            dependency.type_data = crate::type_data::from_syn_type(type_.ty.deref())?;
            binds.dependencies.push(dependency);
        }
    }
    let provides_attr = parsing::get_parenthesized_field_values(&attr.meta)?;
    if let Some(scope) = provides_attr.get("scope") {
        let scopes = parsing::get_types(Some(scope), attr.span())?;
        for scope in &scopes {
            type_validator.add_dyn_type(scope, attr.span());
        }
        for (path, span) in scope.get_paths()? {
            type_validator.add_dyn_path(&path, span);
        }
        binds.type_data.scopes.extend(scopes);
    }
    Ok(binds)
}

fn handle_binds_option_of(
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
        let return_type = crate::type_data::from_syn_type(ty.deref())?;
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
    Ok(binds_option_of)
}

fn handle_multibinds(
    signature: &mut syn::Signature,
    block: &mut syn::Block,
) -> Result<Binding, TokenStream> {
    if !block.stmts.is_empty() {
        return spanned_compile_error(block.span(), "#[multibinds] methods must have empty body");
    }
    let body: syn::Stmt = syn::parse2(quote! { unimplemented!(); }).unwrap();
    block.stmts.push(body);

    let mut binds = Binding::new(Multibinds);
    binds.name = signature.ident.to_string();
    if let syn::ReturnType::Type(ref _token, ref mut ty) = signature.output {
        let return_type = crate::type_data::from_syn_type(ty.deref())?;
        match return_type.path.as_str() {
            "std::vec::Vec" => {}
            "std::collections::HashMap" => {}
            _ => {
                return spanned_compile_error(
                    signature.span(),
                    "#[multibinds] methods must return Vec<T> or HashMap<K,V>",
                )
            }
        }
        binds.type_data = return_type.clone();
    } else {
        return spanned_compile_error(signature.span(), "return type expected");
    }
    if !signature.inputs.is_empty() {
        return spanned_compile_error(
            signature.span(),
            "#[multibinds] method must take no arguments",
        );
    }
    Ok(binds)
}

fn has_lifetime(args: &Punctuated<GenericArgument, Token![,]>) -> bool {
    for arg in args {
        if let GenericArgument::Lifetime(_) = arg {
            return true;
        }
    }
    false
}
