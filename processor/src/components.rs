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

use lazy_static::lazy_static;
use proc_macro2::TokenStream;
use quote::quote_spanned;
use quote::{format_ident, quote, ToTokens};
use syn::spanned::Spanned;
use syn::Attribute;

use crate::error::{spanned_compile_error, CompileError};
use crate::graph;
use crate::manifest::{
    with_manifest, BuilderModules, Component, ComponentType, Dependency, Manifest,
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

pub fn handle_component_attribute(
    attr: TokenStream,
    input: TokenStream,
    component_type: ComponentType,
) -> Result<TokenStream, TokenStream> {
    let span = input.span();
    let mut item_trait: syn::ItemTrait =
        syn::parse2(input).map_spanned_compile_error(span, "trait expected")?;
    let mut provisions = Vec::<Dependency>::new();
    let mut type_validator = TypeValidator::new();
    for item in &mut item_trait.items {
        if let syn::TraitItem::Method(ref mut method) = item {
            let mut provision = Dependency::new();
            let mut qualifier: Option<TypeData> = None;
            let mut new_attrs: Vec<Attribute> = Vec::new();
            for attr in &method.attrs {
                match parsing::get_attribute(attr).as_str() {
                    "qualified" => {
                        qualifier = Some(parsing::get_parenthesized_type(&attr.tokens)?);
                        type_validator.add_type(qualifier.as_ref().unwrap(), attr.span());
                    }
                    _ => new_attrs.push(attr.clone()),
                }
            }
            method.attrs = new_attrs;
            provision.name = method.sig.ident.to_string();
            if let syn::ReturnType::Type(ref _token, ref ty) = method.sig.output {
                if is_trait_object_without_lifetime(ty.deref()) {
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
    let attributes = parsing::get_attribute_field_values(attr.clone())?;

    for key in attributes.keys() {
        if !COMPONENT_METADATA_KEYS.contains(key) {
            return spanned_compile_error(attr.span(), &format!("unknown key: {}", key));
        }
    }

    let builder_modules = if let Some(value) = attributes.get("builder_modules") {
        if let FieldValue::Path(span, ref path) = value {
            let type_ = TypeData::from_path_with_span(path.borrow(), span.clone())?;
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
    let identifier = component.type_data.identifier().to_string();

    let subcomponent_builder = if component.component_type == ComponentType::Subcomponent {
        let subcomponent_name = item_trait.ident.clone();
        let builder_name = format_ident!("{}Builder", subcomponent_name);
        let args = if builder_modules.is_some() {
            let args_type = builder_modules.as_ref().unwrap().syn_type();
            quote! {builder_modules: #args_type}
        } else {
            quote! {}
        };
        quote! {
            pub trait #builder_name<'a> {
                fn build(&self, #args) -> ::lockjaw::ComponentLifetime<'a, dyn #subcomponent_name<'a>>;
            }
        }
    } else {
        quote! {}
    };

    with_manifest(|mut manifest| manifest.components.push(component));

    let prologue_check = prologue_check(item_trait.span());
    let validate_type = type_validator.validate(identifier);
    let result = quote! {
        #item_trait
        #subcomponent_builder
        #validate_type
        #prologue_check
    };
    Ok(result)
}

fn is_trait_object_without_lifetime(ty: &syn::Type) -> bool {
    let tokens: Vec<String> = ty
        .to_token_stream()
        .into_iter()
        .map(|t| t.to_string())
        .collect();
    if !tokens.contains(&"dyn".to_owned()) {
        return false;
    }
    !tokens.contains(&"'".to_owned())
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
            .map_spanned_compile_error(span, "tuples module manifests cannot have builders")?;
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

pub fn generate_components(manifest: &Manifest) -> Result<(TokenStream, Vec<String>), TokenStream> {
    let mut result = quote! {};
    let mut messages = Vec::<String>::new();
    for component in &manifest.components {
        if component
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
