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

use crate::environment;
use crate::error::spanned_compile_error;
use crate::manifest::{with_manifest, ExpandedVisibility, Manifest, TypeRoot};
use crate::type_data::TypeData;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{ItemStruct, ItemTrait, VisPublic, Visibility};

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
    item_struct.vis = Visibility::Public(VisPublic {
        pub_token: syn::token::Pub(item_struct.span()),
    });

    let type_ = TypeData::from_local(&original_ident.to_string(), original_ident.span())?;
    let crate_type = TypeData::from_local(&exported_ident.to_string(), original_ident.span())?;

    with_manifest(|mut manifest| {
        let mut exported_type = TypeData::new();
        exported_type.root = TypeRoot::CRATE;
        exported_type.path = type_.identifier().to_string();
        exported_type.field_crate = environment::current_crate();

        manifest.expanded_visibilities.insert(
            type_.canonical_string_path(),
            ExpandedVisibility {
                crate_local_name: crate_type,
                exported_name: exported_type,
            },
        );
    });

    Ok(quote! {
        #[allow(non_camel_case_types)]
        #item_struct

        #original_vis use #exported_ident as #original_ident;
    })
}

fn handle_item_trait(mut item_trait: ItemTrait) -> Result<TokenStream, TokenStream> {
    let original_ident = item_trait.ident.clone();
    let original_vis = item_trait.vis.clone();
    let exported_ident = format_ident!("lockjaw_export_type_{}", original_ident);

    item_trait.ident = exported_ident.clone();
    item_trait.vis = Visibility::Public(VisPublic {
        pub_token: syn::token::Pub(item_trait.span()),
    });

    let mut type_ = TypeData::from_local(&original_ident.to_string(), original_ident.span())?;
    type_.trait_object = true;
    let crate_type = TypeData::from_local(&exported_ident.to_string(), original_ident.span())?;

    with_manifest(|mut manifest| {
        let mut exported_type = TypeData::new();
        exported_type.root = TypeRoot::CRATE;
        exported_type.path = type_.identifier().to_string();
        exported_type.field_crate = environment::current_crate();
        exported_type.trait_object = true;

        manifest.expanded_visibilities.insert(
            type_.canonical_string_path(),
            ExpandedVisibility {
                crate_local_name: crate_type,
                exported_name: exported_type,
            },
        );
    });

    Ok(quote! {
        #[allow(non_camel_case_types)]
        #item_trait

        #original_vis use #exported_ident as #original_ident;
    })
}

pub fn expand_visibilities(manifest: &Manifest) -> Result<TokenStream, TokenStream> {
    let mut result = quote! {};
    for expanded_visibility in &manifest.expanded_visibilities {
        let local_type = expanded_visibility.1.crate_local_name.syn_type();
        let exported_type = format_ident!("{}", expanded_visibility.1.exported_name.path);
        result = quote! {
            #result
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
