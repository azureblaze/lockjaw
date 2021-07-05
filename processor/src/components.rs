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
use std::cell::RefCell;
use std::collections::HashSet;
use std::ops::Deref;

use lazy_static::lazy_static;
use proc_macro2::TokenStream;
use quote::quote_spanned;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;

use crate::error::{spanned_compile_error, CompileError};
use crate::graph;
use crate::manifest::{Component, ComponentModuleManifest, Dependency, Manifest, TypeRoot};
use crate::type_data::TypeData;
use crate::{environment, parsing};
use syn::Attribute;

thread_local! {
    static COMPONENTS :RefCell<Vec<LocalComponent>> = RefCell::new(Vec::new());
}

thread_local! {
    static COMPONENT_MODULE_MANIFESTS :RefCell<Vec<LocalComponentModuleManifest>> = RefCell::new(Vec::new());
}

lazy_static! {
    static ref COMPONENT_METADATA_KEYS: HashSet<String> = {
        let mut set = HashSet::<String>::new();
        set.insert("modules".to_owned());
        set.insert("path".to_owned());
        set
    };
}

/// Stores partial data until the true path can be resolved in the file epilogue.
struct LocalComponent {
    name: String,
    provisions: Vec<Dependency>,
    additional_path: Option<String>,
    module_manifest: Option<TypeData>,
}

/// Stores partial data until the true path can be resolved in the file epilogue.
struct LocalComponentModuleManifest {
    name: String,
    additional_path: Option<String>,
    builder_modules: Vec<Dependency>,
    modules: Vec<TypeData>,
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
        module_manifest = Some(TypeData::from_path(path.borrow())?);
    } else {
        module_manifest = Option::None;
    }

    let component = LocalComponent {
        name: item_trait.ident.to_string(),
        provisions,
        additional_path: attributes.get("path").cloned(),
        module_manifest,
    };

    COMPONENTS.with(|components| components.borrow_mut().push(component));

    let result = quote! {
        #item_trait
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

pub fn generate_component_manifest(base_path: &str) -> Vec<Component> {
    COMPONENTS.with(|c| {
        let mut components = c.borrow_mut();
        let mut result = Vec::<Component>::new();
        for local_component in components.iter() {
            let mut component = Component::new();
            let mut type_ = TypeData::new();
            type_.field_crate = environment::current_crate();
            type_.root = TypeRoot::CRATE;
            let mut path = String::new();
            if !base_path.is_empty() {
                path.push_str(base_path);
                path.push_str("::");
            }
            if let Some(additional_path) = &local_component.additional_path {
                path.push_str(additional_path);
                path.push_str("::");
            }
            path.push_str(&local_component.name);

            type_.path = path;
            component.type_data = type_;
            component
                .provisions
                .extend(local_component.provisions.clone());
            if let Some(ref m) = local_component.module_manifest {
                component.module_manifest = Some(m.clone());
            }
            result.push(component);
        }
        components.clear();
        result
    })
}

pub fn handle_component_module_manifest_attribute(
    attr: TokenStream,
    input: TokenStream,
) -> Result<TokenStream, TokenStream> {
    let span = input.span();
    let item_struct: syn::ItemStruct =
        syn::parse2(input).map_spanned_compile_error(span, "struct expected")?;
    let attributes = parsing::get_attribute_metadata(attr.clone())?;
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
    let manifest = LocalComponentModuleManifest {
        name: item_struct.ident.to_string(),
        additional_path: attributes.get("path").cloned(),
        builder_modules,
        modules,
    };

    COMPONENT_MODULE_MANIFESTS.with(|c| c.borrow_mut().push(manifest));

    let vis = item_struct.vis;
    let ident = item_struct.ident;
    Ok(quote_spanned! {span=>
        #vis struct #ident {
            #fields
        }
    })
}

pub fn generate_component_module_manifest(base_path: &str) -> Vec<ComponentModuleManifest> {
    COMPONENT_MODULE_MANIFESTS.with(|c| {
        let mut components_module_manifests = c.borrow_mut();
        let mut result = Vec::<ComponentModuleManifest>::new();
        for local_component_module_manifest in components_module_manifests.iter() {
            let mut component_module_manifest = ComponentModuleManifest::new();
            component_module_manifest.type_data = Some(TypeData::from_local(
                base_path,
                &local_component_module_manifest.additional_path,
                &local_component_module_manifest.name,
            ));
            component_module_manifest
                .modules
                .extend(local_component_module_manifest.modules.clone());
            component_module_manifest
                .builder_modules
                .extend(local_component_module_manifest.builder_modules.clone());
            result.push(component_module_manifest);
        }
        components_module_manifests.clear();
        result
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
