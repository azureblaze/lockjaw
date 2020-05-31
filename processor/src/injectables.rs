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

use crate::error::CompileError;
#[allow(unused)]
use crate::log;
use crate::manifests::type_from_syn_type;
use crate::protos::manifest::{Field, Injectable, Manifest, Type, Type_Root};
use crate::{environment, manifests, parsing};
use proc_macro2::TokenStream;
use quote::quote;
use quote::{format_ident, ToTokens};
use std::borrow::Borrow;
use std::cell::RefCell;
use syn::export::TokenStream2;
use syn::spanned::Spanned;

struct LocalInjectable {
    identifier: String,
    //span: proc_macro2::Span,
    additional_path: Option<String>,
    scopes: Vec<Type>,
    fields: Vec<Field>,
}

thread_local! {
    static INJECTABLES :RefCell<Vec<LocalInjectable>> = RefCell::new(Vec::new());
}

pub fn handle_injectable_attribute(
    attr: TokenStream,
    input: TokenStream,
) -> Result<TokenStream, TokenStream> {
    let span = input.span();
    let mut item: syn::ItemStruct =
        syn::parse2(input).map_spanned_compile_error(span, "struct expected")?;

    let attributes = parsing::get_attributes(attr)?;
    let scopes = parsing::get_types(attributes.get("scope").map(Clone::clone))?;
    let mut injectable = LocalInjectable {
        identifier: item.ident.to_string(),
        //span: item.ident.span().clone(),
        additional_path: attributes.get("path").cloned(),
        scopes,
        fields: Vec::new(),
    };

    for mut field in item.fields.iter_mut() {
        let mut proto_field = Field::new();
        proto_field.set_name(
            field
                .ident
                .as_ref()
                .map_spanned_compile_error(field.span(), "tuple injectable not supported")?
                .to_string(),
        );
        proto_field.set_field_type(type_from_syn_type(field.ty.borrow())?);

        let mut new_attrs: Vec<syn::Attribute> = Vec::new();
        for attr in &field.attrs {
            if parsing::is_attribute(attr, "inject") {
                proto_field.set_injected(true);
            } else {
                new_attrs.push(attr.clone());
            }
        }
        field.attrs = new_attrs;
        injectable.fields.push(proto_field);
    }

    INJECTABLES.with(|injectables| {
        injectables.borrow_mut().push(injectable);
    });
    Ok(item.to_token_stream())
}

pub fn generate_injectables(manifest: &Manifest) -> TokenStream {
    let mut result = TokenStream2::new();
    for injectable in manifest.get_injectables() {
        if injectable
            .get_field_crate()
            .ne(&environment::current_crate())
        {
            continue;
        }
        let tokens = generate_injectable(injectable);
        result = quote! {
            #result
            #tokens
        };
    }
    result
}

fn generate_injectable(injectable: &Injectable) -> TokenStream {
    let mut has_ref = false;
    let mut params = quote! {};
    for field in injectable.get_fields() {
        if field.get_injected() {
            let param_name = format_ident!("{}", field.get_name());
            let param_type = field.get_field_type().local_path();
            if field.get_field_type().get_field_ref() {
                params = quote! {
                   #params #param_name : &'a #param_type,
                };
                has_ref = true;
            } else {
                params = quote! {
                   #params #param_name : #param_type,
                }
            }
        }
    }
    let mut ctor_params = quote! {};
    for field in &injectable.fields {
        let param_name = format_ident!("{}", field.get_name());
        if field.get_injected() {
            ctor_params = quote! {
               #ctor_params #param_name,
            }
        } else {
            let param_type = field.get_field_type().local_path();
            ctor_params = quote! {
               #ctor_params
               #param_name : <#param_type>::default(),
            }
        }
    }
    let name = injectable.get_field_type().local_path();
    let lifetime;
    if has_ref {
        lifetime = quote! {<'a>};
    } else {
        lifetime = quote! {};
    }
    let result = quote! {
        impl #lifetime #name #lifetime {
            pub fn lockjaw_new(#params) -> Self {
               #name{#ctor_params}
            }
        }
    };
    result
}

pub fn generate_manifest(base_path: &str) -> Vec<Injectable> {
    INJECTABLES.with(|injectables| {
        let mut result = Vec::new();
        for local_injectable in injectables.borrow().iter() {
            let mut injectable = Injectable::new();
            let mut type_ = Type::new();
            type_.set_field_crate(environment::current_crate());
            type_.set_root(Type_Root::CRATE);
            manifests::extend(type_.mut_scopes(), local_injectable.scopes.clone());
            let mut path = String::new();
            if !base_path.is_empty() {
                path.push_str(base_path);
                path.push_str("::");
            }
            if let Some(additional_path) = &local_injectable.additional_path {
                path.push_str(additional_path);
                path.push_str("::");
            }
            path.push_str(&local_injectable.identifier);

            type_.set_path(path);
            injectable.set_field_type(type_);
            injectable.set_field_crate(environment::current_crate());
            for local_field in &local_injectable.fields {
                injectable.mut_fields().push(local_field.clone());
            }

            result.push(injectable);
        }
        injectables.borrow_mut().clear();
        result
    })
}
