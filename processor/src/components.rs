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

use crate::error::{spanned_compile_error, CompileError};
use crate::graph;
use crate::parsing;
use crate::parsing::FieldValue;
use crate::type_data::ProcessorTypeData;
use crate::type_validator::TypeValidator;
use base64::engine::Engine;
use lazy_static::lazy_static;
use lockjaw_common::manifest::{ComponentType, Manifest};
use lockjaw_common::type_data::TypeData;
use proc_macro2::{Ident, TokenStream};
use quote::quote_spanned;
use quote::{format_ident, quote, ToTokens};
use syn::spanned::Spanned;
use syn::{Attribute, ItemTrait};

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
) -> Result<TokenStream, TokenStream> {
    let span = input.span();
    let mut item_trait: syn::ItemTrait =
        syn::parse2(input).map_spanned_compile_error(span, "trait expected")?;

    let mut type_validator = TypeValidator::new();

    parse_provisions(&mut item_trait, &mut type_validator)?;

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
            type_validator.add_path(path, span.clone());
            Some(path)
        } else {
            return spanned_compile_error(value.span(), "path expected for modules");
        }
    } else {
        None
    };

    if let Some(value) = attributes.get("modules") {
        match value {
            FieldValue::Path(span, ref path) => {
                type_validator.add_path(path, span.clone());
            }
            FieldValue::Array(span, ref array) => {
                for field in array {
                    if let FieldValue::Path(span, ref path) = field {
                        type_validator.add_path(path, span.clone());
                    } else {
                        return spanned_compile_error(span.clone(), "path expected for modules");
                    }
                }
            }
            _ => {
                return spanned_compile_error(value.span(), "path expected for modules");
            }
        }
    }

    let component_vis = item_trait.vis.clone();

    let component_builder = if component_type == ComponentType::Subcomponent {
        let subcomponent_name = item_trait.ident.clone();
        let builder_name = format_ident!("{}Builder", subcomponent_name);
        let args = if let Some(args_type) = builder_modules {
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
        let component_name = item_trait.ident.clone();
        let address_ident = format_ident!("LOCKJAW_COMPONENT_BUILDER_ADDR_{}", item_trait.ident);
        let components_initializer_name = format_ident!("lockjaw_init_root_components");

        if let Some(module_manifest_name) = builder_modules {
            quote! {
                #[doc(hidden)]
                #[allow(non_upper_case_globals)]
                pub static mut #address_ident : *const () = ::std::ptr::null();

                impl dyn #component_name {

                    #[allow(unused)]
                    pub fn build (param : #module_manifest_name) -> Box<dyn #component_name>{
                        extern "Rust" {
                                fn  #components_initializer_name();
                        }
                        unsafe {
                            #components_initializer_name();
                            let builder: extern "Rust" fn(param : #module_manifest_name) -> Box<dyn #component_name> = std::mem::transmute(#address_ident);
                            builder(param)
                        }
                    }
                }
            }
        } else {
            quote! {
                #[doc(hidden)]
                #[allow(non_upper_case_globals)]
                pub static mut #address_ident : *const () = ::std::ptr::null();

                impl dyn #component_name {
                    pub fn build () -> Box<dyn #component_name>{
                        extern "Rust" {
                                fn  #components_initializer_name();
                        }
                        unsafe{
                            #components_initializer_name();
                            let builder: extern "Rust" fn() -> Box<dyn #component_name> = std::mem::transmute(#address_ident);
                            builder()
                        }
                    }
                    pub fn new () -> Box<dyn #component_name>{
                        extern "Rust" {
                            fn  #components_initializer_name();
                        }
                        unsafe{
                            #components_initializer_name();
                            let builder: extern "Rust" fn() -> Box<dyn #component_name> = std::mem::transmute(#address_ident);
                            builder()
                        }
                    }
                }
            }
        }
    };

    let parent_module = if let Some(parent) = attributes.get("parent") {
        if let FieldValue::Path(_, path) = parent {
            let module_name =
                format_ident!("lockjaw_parent_module_{}", item_trait.ident.to_string());
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

    let validate_type = type_validator.validate(item_trait.ident.to_string());
    let result = quote! {
        #item_trait
        #component_builder
        #parent_module
        #validate_type
    };
    Ok(result)
}

pub fn builder_name(component: &TypeData) -> Ident {
    format_ident!(
        "lockjaw_component_builder_{}",
        base64::prelude::BASE64_STANDARD_NO_PAD
            .encode(format!("{}", component.identifier().to_string(),))
            .replace("+", "_P")
            .replace("/", "_S")
    )
}

pub fn parse_provisions(
    item_trait: &mut ItemTrait,
    type_validator: &mut TypeValidator,
) -> Result<(), TokenStream> {
    for item in &mut item_trait.items {
        if let syn::TraitItem::Fn(ref mut method) = item {
            let mut new_attrs: Vec<Attribute> = Vec::new();
            for attr in &method.attrs {
                match parsing::get_attribute(attr).as_str() {
                    "qualified" => {
                        let tokens = attr.meta.require_list().unwrap().tokens.to_token_stream();
                        let path = parsing::get_path(&tokens)?;
                        type_validator.add_path(&path, path.span());
                    }
                    _ => new_attrs.push(attr.clone()),
                }
            }
            method.attrs = new_attrs;
            let syn::ReturnType::Type(_, _) = method.sig.output else {
                return spanned_compile_error(
                    method.sig.span(),
                    "return type expected for component provisions",
                );
            };
        }
    }
    Ok(())
}

pub fn handle_builder_modules_attribute(
    _attr: TokenStream,
    input: TokenStream,
) -> Result<TokenStream, TokenStream> {
    let span = input.span();
    let item_struct: syn::ItemStruct =
        syn::parse2(input).map_spanned_compile_error(span, "struct expected")?;

    for field in &item_struct.fields {
        let span = field.span();
        field
            .ident
            .as_ref()
            .map_spanned_compile_error(span, "#[builder_modules] cannot be tuples")?;
    }

    Ok(quote_spanned! {span=>
        #item_struct
    })
}

pub fn generate_components(
    manifest: &Manifest,
    root: bool,
) -> Result<(TokenStream, TokenStream, Vec<String>), TokenStream> {
    let mut result = quote! {};
    let mut initializer = quote! {};
    let mut messages = Vec::<String>::new();
    for component in &manifest.components {
        if component.definition_only {
            if !root {
                continue;
            }
        }
        if component.component_type != ComponentType::Component {
            continue;
        }
        let (tokens, message) = graph::generate_component(&component, manifest)?;
        result = quote! {
            #result
            #tokens
        };
        let component_initialzer =
            format_ident!("lockjaw_init_{}", component.type_data.identifier_string());
        initializer = quote! {
            #initializer
            #component_initialzer();
        };
        messages.push(message);
    }
    //log!("{}", result.to_string());
    Ok((result, initializer, messages))
}
