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
#[allow(unused)]
use crate::log;
use crate::manifests::{type_from_path, type_from_syn_type};
use crate::protos::manifest::{
    Component, ComponentModuleManifest, Dependency, Manifest, Type, Type_Root,
};
use crate::{environment, parsing};
use crate::{graph, manifests};
use proc_macro2::TokenStream;
use quote::quote;
use quote::quote_spanned;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::ops::Deref;
use syn::spanned::Spanned;

thread_local! {
    static COMPONENTS :RefCell<Vec<LocalComponent>> = RefCell::new(Vec::new());
}

thread_local! {
    static COMPONENT_MODULE_MANIFESTS :RefCell<Vec<LocalComponentModuleManifest>> = RefCell::new(Vec::new());
}

/// Stores partial data until the true path can be resolved in the file epilogue.
struct LocalComponent {
    name: String,
    provisions: Vec<Dependency>,
    additional_path: Option<String>,
    module_manifest: Option<Type>,
}

/// Stores partial data until the true path can be resolved in the file epilogue.
struct LocalComponentModuleManifest {
    name: String,
    additional_path: Option<String>,
    builder_modules: Vec<Dependency>,
    modules: Vec<Type>,
}

pub fn handle_component_attribute(
    attr: TokenStream,
    input: TokenStream,
) -> Result<TokenStream, TokenStream> {
    let span = input.span();
    let item_trait: syn::ItemTrait =
        syn::parse2(input).map_spanned_compile_error(span, "trait expected")?;
    let mut provisions = Vec::<Dependency>::new();
    for item in &item_trait.items {
        if let syn::TraitItem::Method(ref method) = item {
            let mut provision = Dependency::new();
            provision.set_name(method.sig.ident.to_string());
            if let syn::ReturnType::Type(ref _token, ref ty) = method.sig.output {
                provision.set_field_type(type_from_syn_type(ty.deref())?);
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
    let attributes = parsing::get_attributes(attr.clone())?;
    if let Some(value) = attributes.get("modules") {
        let path: syn::Path = syn::parse_str(value)
            .map_spanned_compile_error(attr.span(), "path expected for modules")?;
        module_manifest = Some(type_from_path(path.borrow())?);
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

pub fn generate_component_manifest(base_path: &str) -> Vec<Component> {
    COMPONENTS.with(|c| {
        let mut components = c.borrow_mut();
        let mut result = Vec::<Component>::new();
        for local_component in components.iter() {
            let mut component = Component::new();
            let mut type_ = Type::new();
            type_.set_field_crate(environment::current_crate());
            type_.set_root(Type_Root::CRATE);
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

            type_.set_path(path);
            component.set_field_type(type_);
            manifests::extend(
                component.mut_provisions(),
                local_component.provisions.clone(),
            );
            if let Some(ref m) = local_component.module_manifest {
                component.set_module_manifest(m.clone());
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
    let attributes = parsing::get_attributes(attr.clone())?;
    let mut builder_modules = <Vec<Dependency>>::new();
    let mut modules = <Vec<Type>>::new();
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
            dep.set_name(name.to_string());
            dep.set_field_type(type_from_syn_type(field.ty.borrow())?);
            builder_modules.push(dep);

            let ty = field.ty;
            fields = quote! {
                #fields
                #name : #ty,
            }
        } else {
            modules.push(type_from_syn_type(field.ty.borrow())?)
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
            let mut type_ = Type::new();
            type_.set_field_crate(environment::current_crate());
            type_.set_root(Type_Root::CRATE);
            let mut path = String::new();
            if !base_path.is_empty() {
                path.push_str(base_path);
                path.push_str("::");
            }
            if let Some(additional_path) = &local_component_module_manifest.additional_path {
                path.push_str(additional_path);
                path.push_str("::");
            }
            path.push_str(&local_component_module_manifest.name);

            type_.set_path(path);
            component_module_manifest.set_field_type(type_);
            manifests::extend(
                component_module_manifest.mut_modules(),
                local_component_module_manifest.modules.clone(),
            );
            manifests::extend(
                component_module_manifest.mut_builder_modules(),
                local_component_module_manifest.builder_modules.clone(),
            );
            result.push(component_module_manifest);
        }
        components_module_manifests.clear();
        result
    })
}

pub fn generate_components(manifest: &Manifest) -> Result<TokenStream, TokenStream> {
    let mut result = quote! {};
    for component in manifest.get_components() {
        if component
            .get_field_type()
            .get_field_crate()
            .ne(&environment::current_crate())
        {
            continue;
        }
        let tokens = graph::generate_component(component, manifest)?;
        result = quote! {
            #result
            #tokens
        }
    }
    //log!("{}", result.to_string());
    Ok(result)
}
