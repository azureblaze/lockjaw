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
use quote::quote;
use syn::parse_quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Attribute, GenericArgument};
use syn::{ImplItemFn, Token};

use crate::error::{spanned_compile_error, CompileError};
use crate::parsing;
use crate::parsing::{get_parenthesized_field_values, FieldValue};

use crate::type_validator::TypeValidator;
use lockjaw_common::manifest::{BindingType, MultibindingType};

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

    let mut item_impl: syn::ItemImpl =
        syn::parse2(input.clone()).map_spanned_compile_error(span, "impl expected")?;
    let syn::Type::Path(_) = item_impl.self_ty.deref() else {
        return spanned_compile_error(item_impl.span(), "path expected");
    };
    let mut type_validator = TypeValidator::new();
    for i in 0..item_impl.items.len() {
        #[allow(unused_mut)] // required
        let mut item = item_impl.items.get_mut(i).unwrap();
        if let syn::ImplItem::Fn(ref mut method) = item {
            parse_binding(method, &mut type_validator)?;
        }
    }

    if let Some(subcomponents) = attributes.get("subcomponents") {
        let paths = subcomponents.get_paths()?;
        for (path, span) in &paths {
            type_validator.add_dyn_path(path, span.clone());
        }
    }
    if let Some(install_in) = attributes.get("install_in") {
        let paths = install_in.get_paths()?;
        for (path, span) in &paths {
            type_validator.add_dyn_path(path, span.clone());
        }
    }

    let validate_type = type_validator.validate(parsing::type_string(&item_impl.self_ty)?);

    let result = quote! {
        #item_impl
        #validate_type
    };
    Ok(result)
}

fn parse_binding(
    method: &mut ImplItemFn,
    type_validator: &mut TypeValidator,
) -> Result<(), TokenStream> {
    let mut option_binding: Option<BindingType> = None;
    let mut multibinding = MultibindingType::None;
    let mut new_attrs: Vec<syn::Attribute> = Vec::new();
    for attr in &method.attrs {
        let attr_str = parsing::get_attribute(attr);
        match attr_str.as_str() {
            "provides" => {
                if option_binding.is_some() {
                    return spanned_compile_error(attr.span(), "#[module] methods can only be annotated by one of #[provides]/#[binds]/#[binds_option_of]/#[multibinds]");
                }
                handle_provides(attr, &mut method.sig, type_validator)?;
                option_binding = Some(BindingType::Provides);
            }
            "binds" => {
                if option_binding.is_some() {
                    return spanned_compile_error(attr.span(), "#[module] methods can only be annotated by one of #[provides]/#[binds]/#[binds_option_of]/#[multibinds]");
                }
                handle_binds(attr, &mut method.sig, &mut method.block, type_validator)?;
                option_binding = Some(BindingType::Binds);
                let allow_dead_code: Attribute = parse_quote! {#[allow(dead_code)]};
                new_attrs.push(allow_dead_code);
                let allow_unused: Attribute = parse_quote! {#[allow(unused)]};
                new_attrs.push(allow_unused);
            }
            "binds_option_of" => {
                if option_binding.is_some() {
                    return spanned_compile_error(attr.span(), "#[module] methods can only be annotated by one of #[provides]/#[binds]/#[binds_option_of]/#[multibinds]");
                }
                handle_binds_option_of(&mut method.sig, &mut method.block)?;
                option_binding = Some(BindingType::BindsOptionOf);
                let allow_dead_code: Attribute = parse_quote! {#[allow(dead_code)]};
                new_attrs.push(allow_dead_code);
            }
            "multibinds" => {
                if option_binding.is_some() {
                    return spanned_compile_error(attr.span(), "#[module] methods can only be annotated by one of #[provides]/#[binds]/#[binds_option_of]/#[multibinds]");
                }
                handle_multibinds(&mut method.sig, &mut method.block)?;
                option_binding = Some(BindingType::Multibinds);
                let allow_dead_code: Attribute = parse_quote! {#[allow(dead_code)]};
                new_attrs.push(allow_dead_code);
            }
            "into_vec" => {
                multibinding = MultibindingType::IntoVec;
            }
            "elements_into_vec" => {
                multibinding = MultibindingType::ElementsIntoVec;
                let syn::ReturnType::Type(_, _) = method.sig.output else {
                    return spanned_compile_error(method.sig.span(), "return type expected");
                };
            }
            "qualified" => {}
            "into_map" => {
                multibinding = MultibindingType::IntoMap;
                let fields = get_parenthesized_field_values(&attr.meta)?;
                if let Some(field) = fields.get("string_key") {
                    let FieldValue::StringLiteral(_, _) = field else {
                        return spanned_compile_error(
                            attr.span(),
                            "string literal expected for string_key",
                        );
                    };
                } else if let Some(field) = fields.get("i32_key") {
                    let FieldValue::IntLiteral(_, _) = field else {
                        return spanned_compile_error(
                            attr.span(),
                            "i32 literal expected for i32_key",
                        );
                    };
                } else if let Some(field) = fields.get("enum_key") {
                    let FieldValue::Path(_, _) = field else {
                        return spanned_compile_error(attr.span(), "path expected for enum_key");
                    };
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
    let binding = option_binding.unwrap();
    if binding == BindingType::Binds {
        if multibinding == MultibindingType::ElementsIntoVec {
            return spanned_compile_error(
                method.span(),
                "#[elements_into_set] cannot be used on #[binds]",
            );
        }
    }
    Ok(())
}

fn handle_provides(
    attr: &syn::Attribute,
    signature: &mut syn::Signature,
    type_validator: &mut TypeValidator,
) -> Result<(), TokenStream> {
    let syn::ReturnType::Type(ref _token, _) = signature.output else {
        return spanned_compile_error(signature.span(), "return type expected");
    };
    for args in &signature.inputs {
        match args {
            syn::FnArg::Receiver(ref receiver) => {
                if receiver.reference.is_none() {
                    return spanned_compile_error(args.span(), "modules should not consume self");
                }
            }
            syn::FnArg::Typed(ref type_) => {
                let syn::Pat::Ident(_) = type_.pat.deref() else {
                    return spanned_compile_error(args.span(), "identifier expected");
                };
            }
        }
    }
    let provides_attr = parsing::get_parenthesized_field_values(&attr.meta)?;
    if let Some(scope) = provides_attr.get("scope") {
        for (path, span) in scope.get_paths()? {
            type_validator.add_dyn_path(&path, span);
        }
    }
    Ok(())
}

fn handle_binds(
    attr: &syn::Attribute,
    signature: &mut syn::Signature,
    block: &mut syn::Block,
    type_validator: &mut TypeValidator,
) -> Result<(), TokenStream> {
    if !block.stmts.is_empty() {
        return spanned_compile_error(block.span(), "#[binds] methods must have empty body");
    }
    let body: syn::Stmt = syn::parse2(quote! { unimplemented!(); }).unwrap();
    block.stmts.push(body);

    if let syn::ReturnType::Type(ref _token, ref mut ty) = signature.output {
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
            let syn::Pat::Ident(_) = type_.pat.deref() else {
                return spanned_compile_error(args.span(), "identifier expected");
            };
        }
    }
    let provides_attr = parsing::get_parenthesized_field_values(&attr.meta)?;
    if let Some(scope) = provides_attr.get("scope") {
        for (path, span) in scope.get_paths()? {
            type_validator.add_dyn_path(&path, span);
        }
    }
    Ok(())
}

fn handle_binds_option_of(
    signature: &mut syn::Signature,
    block: &mut syn::Block,
) -> Result<(), TokenStream> {
    if !block.stmts.is_empty() {
        return spanned_compile_error(
            block.span(),
            "#[binds_option_of] methods must have empty body",
        );
    }
    let body: syn::Stmt = syn::parse2(quote! { unimplemented!(); }).unwrap();
    block.stmts.push(body);

    if let syn::ReturnType::Type(ref _token, ref mut ty) = signature.output {
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
    } else {
        return spanned_compile_error(signature.span(), "return type expected");
    }
    if signature.inputs.len() != 0 {
        return spanned_compile_error(
            signature.span(),
            "binds_option_of method must only take no parameter",
        );
    }
    Ok(())
}

fn handle_multibinds(
    signature: &mut syn::Signature,
    block: &mut syn::Block,
) -> Result<(), TokenStream> {
    if !block.stmts.is_empty() {
        return spanned_compile_error(block.span(), "#[multibinds] methods must have empty body");
    }
    let body: syn::Stmt = syn::parse2(quote! { unimplemented!(); }).unwrap();
    block.stmts.push(body);

    let syn::ReturnType::Type(_, _) = signature.output else {
        return spanned_compile_error(signature.span(), "return type expected");
    };
    if !signature.inputs.is_empty() {
        return spanned_compile_error(
            signature.span(),
            "#[multibinds] method must take no arguments",
        );
    }
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
