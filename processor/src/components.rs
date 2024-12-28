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

use std::borrow::Borrow;
use std::collections::HashSet;
use std::ops::Deref;

use base64::engine::Engine;
use lazy_static::lazy_static;
use proc_macro2::{Ident, TokenStream};
use quote::quote_spanned;
use quote::{format_ident, quote, ToTokens};
use syn::spanned::Spanned;
use syn::{Attribute, ItemTrait};

use crate::error::{spanned_compile_error, CompileError};
use crate::graph;
use crate::manifest::{
    with_manifest, BuilderModules, Component, ComponentType, Dependency, Manifest, TypeRoot,
};
use crate::parsing::FieldValue;
use crate::prologue::prologue_check;
use crate::type_data::TypeData;
use crate::type_validator::TypeValidator;
use crate::{environment, parsing};

lazy_static! {
    static ref COMPONENT_METADATA_KEYS: HashSet<String> = {
        let mut set = HashSet::<String>::new();
        set.insert("modules".to_owned());
        set.insert("builder_modules".to_owned());
        set
    };
}

lazy_static! {
    static ref SUBCOMPONENT_METADATA_KEYS: HashSet<String> = {
        let mut set = HashSet::<String>::new();
        set.insert("parent".to_owned());
        set
    };
}

pub fn handle_component_attribute(
    attr: TokenStream,
    input: TokenStream,
    component_type: ComponentType,
    definition_only: bool,
) -> Result<TokenStream, TokenStream> {
    let span = input.span();
    let mut item_trait: syn::ItemTrait =
        syn::parse2(input).map_spanned_compile_error(span, "trait expected")?;

    let mut type_validator = TypeValidator::new();

    let provisions = get_provisions(&mut item_trait, &mut type_validator)?;

    let attributes = parsing::get_attribute_field_values(attr.clone())?;
    for key in attributes.keys() {
        if !COMPONENT_METADATA_KEYS.contains(key) {
            if component_type == ComponentType::Subcomponent
                && SUBCOMPONENT_METADATA_KEYS.contains(key)
            {
                continue;
            }
            return spanned_compile_error(attr.span(), &format!("unknown key: {}", key));
        }
    }

    let builder_modules = if let Some(value) = attributes.get("builder_modules") {
        if let FieldValue::Path(span, ref path) = value {
            let type_ = TypeData::from_path_with_span(path, span.clone())?;
            type_validator.add_type(&type_, span.clone());
            Some(type_)
        } else {
            return spanned_compile_error(value.span(), "path expected for modules");
        }
    } else {
        None
    };

    let modules = if let Some(value) = attributes.get("modules") {
        match value {
            FieldValue::Path(span, ref path) => {
                let type_ = TypeData::from_path_with_span(&path, span.clone())?;
                type_validator.add_type(&type_, span.clone());
                Some(vec![type_])
            }
            FieldValue::Array(span, ref array) => {
                let mut result = Vec::new();
                for field in array {
                    if let FieldValue::Path(span, ref path) = field {
                        let type_ = TypeData::from_path_with_span(&path, span.clone())?;
                        type_validator.add_type(&type_, span.clone());
                        result.push(type_)
                    } else {
                        return spanned_compile_error(span.clone(), "path expected for modules");
                    }
                }
                Some(result)
            }
            _ => {
                return spanned_compile_error(value.span(), "path expected for modules");
            }
        }
    } else {
        None
    };

    let mut component = Component::new();
    component.type_data =
        TypeData::from_local(&item_trait.ident.to_string(), item_trait.ident.span())?;
    component.component_type = component_type;
    component.provisions.extend(provisions);
    if let Some(ref m) = builder_modules {
        component.builder_modules = Some(m.clone());
    }
    if let Some(ref m) = modules {
        component.modules = m.clone();
    }
    component.definition_only = definition_only;
    let identifier = component.type_data.identifier().to_string();
    let component_vis = item_trait.vis.clone();

    let component_builder = if component.component_type == ComponentType::Subcomponent {
        let subcomponent_name = item_trait.ident.clone();
        let builder_name = format_ident!("{}Builder", subcomponent_name);
        let args = if builder_modules.is_some() {
            let args_type = builder_modules.as_ref().unwrap().syn_type();
            quote! {builder_modules: #args_type}
        } else {
            quote! {}
        };
        quote! {
            #component_vis trait #builder_name<'a> {
                fn build(&self, #args) -> ::lockjaw::Cl<'a, dyn #subcomponent_name<'a>>;
            }
        }
    } else {
        let builder_name = builder_name(&component);
        let component_name = component.type_data.syn_type();
        if component.builder_modules.is_some() {
            let module_manifest_name = component.builder_modules.as_ref().unwrap().syn_type();
            quote! {
                impl dyn #component_name {
                    #[allow(unused)]
                    pub fn build (param : #module_manifest_name) -> Box<dyn #component_name>{
                        extern "Rust" {
                            fn  #builder_name (param : #module_manifest_name) -> Box<dyn #component_name>;
                        }
                       unsafe { #builder_name(param) }
                    }
                }
            }
        } else {
            quote! {
                impl dyn #component_name {
                    pub fn build () -> Box<dyn #component_name>{
                        extern "Rust" {
                            fn  #builder_name() -> Box<dyn #component_name>;
                        }
                       unsafe { #builder_name() }
                    }
                    pub fn new () -> Box<dyn #component_name>{
                        extern "Rust" {
                            fn  #builder_name() -> Box<dyn #component_name>;
                        }
                       unsafe { #builder_name() }
                    }
                }
            }
        }
    };

    with_manifest(|mut manifest| manifest.components.push(component));

    let parent_module = if let Some(parent) = attributes.get("parent") {
        if let FieldValue::Path(_, path) = parent {
            let module_name = format_ident!("lockjaw_parent_module_{}", identifier);
            let subcomponent_name = item_trait.ident.clone();
            quote! {
                #[doc(hidden)]
                pub struct #module_name;

                #[::lockjaw::module(install_in: #path, subcomponents: #subcomponent_name)]
                impl #module_name{}
            }
        } else {
            return spanned_compile_error(parent.span(), "path expected for parent");
        }
    } else {
        quote! {}
    };

    let prologue_check = prologue_check(item_trait.span());
    let validate_type = type_validator.validate(identifier);
    let result = quote! {
        #item_trait
        #component_builder
        #parent_module
        #validate_type
        #prologue_check
    };
    Ok(result)
}

pub fn builder_name(component: &Component) -> Ident {
    format_ident!(
        "lockjaw_component_builder_{}",
        base64::prelude::BASE64_STANDARD_NO_PAD
            .encode(format!("{}", component.type_data.identifier().to_string(),))
            .replace("+", "_P")
            .replace("/", "_S")
    )
}

pub fn get_provisions(
    item_trait: &mut ItemTrait,
    type_validator: &mut TypeValidator,
) -> Result<Vec<Dependency>, TokenStream> {
    let mut provisions = Vec::<Dependency>::new();
    for item in &mut item_trait.items {
        if let syn::TraitItem::Fn(ref mut method) = item {
            let mut provision = Dependency::new();
            let mut qualifier: Option<TypeData> = None;
            let mut new_attrs: Vec<Attribute> = Vec::new();
            for attr in &method.attrs {
                match parsing::get_attribute(attr).as_str() {
                    "qualified" => {
                        qualifier = Some(parsing::get_type(
                            &attr.meta.require_list().unwrap().tokens,
                        )?);
                        type_validator.add_type(qualifier.as_ref().unwrap(), attr.span());
                    }
                    _ => new_attrs.push(attr.clone()),
                }
            }
            method.attrs = new_attrs;
            provision.name = method.sig.ident.to_string();
            if let syn::ReturnType::Type(ref _token, ref ty) = method.sig.output {
                if is_trait_object_without_lifetime(ty.deref())? {
                    return spanned_compile_error(method.sig.span(), "trait object return type may depend on scoped objects, and must have lifetime bounded by the component ");
                }
                provision.type_data = TypeData::from_syn_type(ty.deref())?;
                provision.type_data.qualifier = qualifier.map(Box::new);
            } else {
                return spanned_compile_error(
                    method.sig.span(),
                    "return type expected for component provisions",
                );
            }
            provisions.push(provision);
        }
    }
    Ok(provisions)
}

fn is_trait_object_without_lifetime(ty: &syn::Type) -> Result<bool, TokenStream> {
    let type_ = TypeData::from_syn_type(ty)?;
    if type_.root == TypeRoot::GLOBAL && type_.path == "lockjaw::Cl" {
        return Ok(false);
    }
    let tokens: Vec<String> = ty
        .to_token_stream()
        .into_iter()
        .map(|t| t.to_string())
        .collect();
    if !tokens.contains(&"dyn".to_owned()) {
        return Ok(false);
    }
    Ok(!tokens.contains(&"'".to_owned()))
}

pub fn handle_builder_modules_attribute(
    _attr: TokenStream,
    input: TokenStream,
) -> Result<TokenStream, TokenStream> {
    let span = input.span();
    let item_struct: syn::ItemStruct =
        syn::parse2(input).map_spanned_compile_error(span, "struct expected")?;
    let mut modules = <Vec<Dependency>>::new();

    for field in &item_struct.fields {
        let mut dep = Dependency::new();
        let span = field.span();
        let name = field
            .ident
            .as_ref()
            .map_spanned_compile_error(span, "#[builder_modules] cannot be tuples")?;
        dep.name = name.to_string();
        dep.type_data = TypeData::from_syn_type(field.ty.borrow())?;
        modules.push(dep);
    }

    let mut builder_modules = BuilderModules::new();
    builder_modules.type_data = Some(TypeData::from_local(
        &item_struct.ident.to_string(),
        item_struct.ident.span(),
    )?);
    builder_modules.builder_modules.extend(modules);
    with_manifest(|mut manifest| manifest.builder_modules.push(builder_modules));

    let prologue_check = prologue_check(item_struct.ident.span());
    Ok(quote_spanned! {span=>
        #item_struct
        #prologue_check
    })
}

pub fn generate_components(
    manifest: &Manifest,
    root: bool,
) -> Result<(TokenStream, Vec<String>), TokenStream> {
    let mut result = quote! {};
    let mut messages = Vec::<String>::new();
    for component in &manifest.components {
        if component.definition_only {
            if !root {
                continue;
            }
        } else if component
            .type_data
            .field_crate
            .ne(&environment::current_crate())
        {
            continue;
        }
        if component.component_type != ComponentType::Component {
            continue;
        }
        let (tokens, message) = graph::generate_component(&component, manifest)?;
        result = quote! {
            #result
            #tokens
        };
        messages.push(message);
    }
    //log!("{}", result.to_string());
    Ok((result, messages))
}
