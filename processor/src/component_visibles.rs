/*
Copyright 2021 Google LLC

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

use crate::error::spanned_compile_error;
use crate::type_data::ProcessorTypeData;
use lockjaw_common::manifest::Manifest;
use lockjaw_common::type_data::TypeData;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{ItemStruct, ItemTrait, Token, Visibility};

pub fn handle_component_visible_attribute(
    _attr: TokenStream,
    input: TokenStream,
) -> Result<TokenStream, TokenStream> {
    if let Ok(item_struct) = syn::parse2::<syn::ItemStruct>(input.clone()) {
        return handle_item_struct(item_struct);
    };

    if let Ok(item_trait) = syn::parse2::<syn::ItemTrait>(input.clone()) {
        return handle_item_trait(item_trait);
    };
    spanned_compile_error(input.span(), "unable to handle the item")
}

fn handle_item_struct(mut item_struct: ItemStruct) -> Result<TokenStream, TokenStream> {
    let original_ident = item_struct.ident.clone();
    let original_vis = item_struct.vis.clone();
    let exported_ident = format_ident!("lockjaw_export_type_{}", original_ident);

    item_struct.ident = exported_ident.clone();
    item_struct.vis = Visibility::Public(Token![pub](item_struct.span()));

    Ok(quote! {
        #original_vis use #exported_ident as #original_ident;

        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        #item_struct
    })
}

fn handle_item_trait(mut item_trait: ItemTrait) -> Result<TokenStream, TokenStream> {
    let original_ident = item_trait.ident.clone();
    let original_vis = item_trait.vis.clone();
    let exported_ident = format_ident!("lockjaw_export_type_{}", original_ident);

    item_trait.ident = exported_ident.clone();
    item_trait.vis = Visibility::Public(syn::token::Pub(item_trait.span()));

    Ok(quote! {
        #original_vis use #exported_ident as #original_ident;

        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        #item_trait
    })
}

pub fn expand_visibilities(manifest: &Manifest) -> Result<TokenStream, TokenStream> {
    let mut result = quote! {};
    for expanded_visibility in &manifest.expanded_visibilities {
        let local_type = expanded_visibility.1.crate_local_name.syn_type();
        let exported_type = format_ident!("{}", expanded_visibility.1.exported_name.path);
        result = quote! {
            #result
            #[doc(hidden)]
            pub use #local_type as #exported_type;
        }
    }
    Ok(result)
}

pub fn visible_type(manifest: &Manifest, type_: &TypeData) -> TypeData {
    if type_.field_ref {
        return visible_ref_type(manifest, type_);
    }
    let mut result = if let Some(ev) = manifest
        .expanded_visibilities
        .get(&type_.canonical_string_path_without_args())
    {
        ev.exported_name.clone()
    } else {
        type_.clone()
    };
    for i in 0..type_.args.len() {
        result.args[i] = visible_type(manifest, &type_.args[i]);
    }
    result
}

pub fn visible_ref_type(manifest: &Manifest, type_: &TypeData) -> TypeData {
    let mut result = type_.clone();
    result.field_ref = false;
    result = visible_type(manifest, &result);
    result.field_ref = true;
    return result;
}
