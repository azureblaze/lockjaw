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
use quote::{quote, ToTokens};
use syn::spanned::Spanned;

use crate::error::{spanned_compile_error, CompileError};
use crate::graph;
use crate::manifest::{with_manifest, Component, ComponentModuleManifest, Dependency, Manifest};
use crate::prologue::prologue_check;
use crate::type_data::TypeData;
use crate::{environment, parsing};
use syn::Attribute;

lazy_static! {
    static ref COMPONENT_METADATA_KEYS: HashSet<String> = {
        let mut set = HashSet::<String>::new();
        set.insert("modules".to_owned());
        set
    };
}

pub fn handle_component_attribute(
    attr: TokenStream,
    input: TokenStream,
) -> Result<TokenStream, TokenStream> {
    let span = input.span();
    let mut item_trait: syn::ItemTrait =
        syn::parse2(input).map_spanned_compile_error(span, "trait expected")?;
    let mut provisions = Vec::<Dependency>::new();
    for item in &mut item_trait.items {
        if let syn::TraitItem::Method(ref mut method) = item {
            let mut provision = Dependency::new();
            let mut qualifier: Option<TypeData> = None;
            let mut new_attrs: Vec<Attribute> = Vec::new();
            for attr in &method.attrs {
                match parsing::get_attribute(attr).as_str() {
                    "qualified" => qualifier = Some(parsing::get_parenthesized_type(&attr.tokens)?),
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
    let module_manifest;
    let attributes = parsing::get_attribute_metadata(attr.clone())?;

    for key in attributes.keys() {
        if !COMPONENT_METADATA_KEYS.contains(key) {
            return spanned_compile_error(attr.span(), &format!("unknown key: {}", key));
        }
    }

    if let Some(value) = attributes.get("modules") {
        let path: syn::Path = syn::parse_str(value)
            .map_spanned_compile_error(attr.span(), "path expected for modules")?;
        module_manifest = Some(TypeData::from_path_with_span(path.borrow(), attr.span())?);
    } else {
        module_manifest = Option::None;
    }

    let mut component = Component::new();
    component.type_data =
        TypeData::from_local(&item_trait.ident.to_string(), item_trait.ident.span())?;
    component.provisions.extend(provisions);
    if let Some(ref m) = module_manifest {
        component.module_manifest = Some(m.clone());
    }

    with_manifest(|mut manifest| manifest.components.push(component));

    let prologue_check = prologue_check(item_trait.span());
    let result = quote! {
        #item_trait
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

pub fn handle_component_module_manifest_attribute(
    _attr: TokenStream,
    input: TokenStream,
) -> Result<TokenStream, TokenStream> {
    let span = input.span();
    let item_struct: syn::ItemStruct =
        syn::parse2(input).map_spanned_compile_error(span, "struct expected")?;
    let mut builder_modules = <Vec<Dependency>>::new();
    let mut modules = <Vec<TypeData>>::new();
    let mut fields = quote! {};

    for field in item_struct.fields {
        let mut is_builder = false;
        for attr in &field.attrs {
            if parsing::is_attribute(&attr, "builder") {
                is_builder = true;
            } else {
                return spanned_compile_error(attr.span(), "lockjaw::component_module_manifest struct fields can only have 'builder' attribute");
            }
        }

        if is_builder {
            let mut dep = Dependency::new();
            let span = field.span().clone();
            let name = field
                .ident
                .map_spanned_compile_error(span, "tuples module manifests cannot have builders")?;
            dep.name = name.to_string();
            dep.type_data = TypeData::from_syn_type(field.ty.borrow())?;
            builder_modules.push(dep);

            let ty = field.ty;
            fields = quote! {
                #fields
                #name : #ty,
            }
        } else {
            modules.push(TypeData::from_syn_type(field.ty.borrow())?)
        }
    }

    let mut component_module_manifest = ComponentModuleManifest::new();
    component_module_manifest.type_data = Some(TypeData::from_local(
        &item_struct.ident.to_string(),
        item_struct.ident.span(),
    )?);
    component_module_manifest.modules.extend(modules);
    component_module_manifest
        .builder_modules
        .extend(builder_modules);
    with_manifest(|mut manifest| {
        manifest
            .component_module_manifests
            .push(component_module_manifest)
    });

    let vis = item_struct.vis;
    let ident = item_struct.ident;
    let prologue_check = prologue_check(ident.span());
    Ok(quote_spanned! {span=>
        #vis struct #ident {
            #fields
        }
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
