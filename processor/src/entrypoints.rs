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

use crate::error::{spanned_compile_error, CompileError};
use crate::manifest::with_manifest;
use crate::parsing::FieldValue;
use crate::prologue::prologue_check;
use crate::type_data::ProcessorTypeData;
use crate::type_validator::TypeValidator;
use crate::{components, parsing, type_data};
use base64::engine::Engine;
use lazy_static::lazy_static;
use lockjaw_common::environment::current_crate;
use lockjaw_common::manifest::{EntryPoint, ExpandedVisibility, TypeRoot};
use lockjaw_common::type_data::TypeData;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::collections::HashSet;
use syn::spanned::Spanned;
use syn::{Token, Visibility};

lazy_static! {
    static ref ENTRY_POINT_METADATA_KEYS: HashSet<String> = {
        let mut set = HashSet::<String>::new();
        set.insert("install_in".to_owned());
        set
    };
}

pub fn handle_entry_point_attribute(
    attr: TokenStream,
    input: TokenStream,
) -> Result<TokenStream, TokenStream> {
    let span = input.span();
    let mut item_trait: syn::ItemTrait =
        syn::parse2(input).map_spanned_compile_error(span, "trait expected")?;

    let mut type_validator = TypeValidator::new();

    let provisions = components::get_provisions(&mut item_trait, &mut type_validator)?;

    let attributes = parsing::get_attribute_field_values(attr.clone())?;

    for key in attributes.keys() {
        if !ENTRY_POINT_METADATA_KEYS.contains(key) {
            return spanned_compile_error(attr.span(), &format!("unknown key: {}", key));
        }
    }
    let component = if let FieldValue::Path(span, path) =
        attributes.get("install_in").map_spanned_compile_error(
            attr.span(),
            "install_in metadata expected for #[entry_point]",
        )? {
        let c = type_data::from_path_with_span(path, span.clone())?;
        type_validator.add_dyn_type(&c, span.clone());
        type_validator.add_dyn_path(path, span.clone());
        c
    } else {
        return spanned_compile_error(attr.span(), "path expected for install_in");
    };
    let mut entry_point = EntryPoint::new();
    entry_point.type_data =
        crate::type_data::from_local(&item_trait.ident.to_string(), item_trait.ident.span())?;

    entry_point.provisions.extend(provisions);
    entry_point.component = component.clone();

    let original_ident = item_trait.ident.clone();
    let original_vis = item_trait.vis.clone();
    let exported_ident = format_ident!("lockjaw_export_type_{}", original_ident);

    item_trait.ident = exported_ident.clone();
    item_trait.vis = Visibility::Public(Token![pub](item_trait.span()));

    let type_ = crate::type_data::from_local(&original_ident.to_string(), original_ident.span())?;
    let crate_type =
        crate::type_data::from_local(&exported_ident.to_string(), original_ident.span())?;

    with_manifest(|mut manifest| {
        let mut exported_type = TypeData::new();
        exported_type.root = TypeRoot::CRATE;
        exported_type.path = type_.identifier().to_string();
        exported_type.field_crate = current_crate();

        manifest.expanded_visibilities.insert(
            type_.canonical_string_path_without_args(),
            ExpandedVisibility {
                crate_local_name: crate_type,
                exported_name: exported_type,
            },
        );
    });

    with_manifest(|mut manifest| manifest.entry_points.push(entry_point.clone()));

    let identifier = entry_point.type_data.identifier_string();
    let item_ident = item_trait.ident.clone();
    let component_type = component.syn_type();
    let prologue_check = prologue_check(item_trait.span());
    let validate_type = type_validator.validate(identifier);
    let getter_name = getter_name(&entry_point);
    let result = quote! {
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        #item_trait

        #original_vis use #exported_ident as #original_ident;

        #validate_type
        #prologue_check

        impl dyn #item_ident {
            fn get<'a>(component: &'a dyn #component_type) -> &'a dyn #item_ident {
                extern "Rust"{
                    fn #getter_name(component: &dyn #component_type) -> &'static dyn #item_ident;
                }

                unsafe { #getter_name(component) }
            }
        }
    };
    Ok(result)
}

pub fn getter_name(entry_point: &EntryPoint) -> Ident {
    format_ident!(
        "lockjaw_entry_point_getter_{}",
        base64::prelude::BASE64_STANDARD_NO_PAD
            .encode(format!(
                "{}_{}",
                entry_point.type_data.identifier().to_string(),
                entry_point.component.identifier().to_string()
            ))
            .replace("+", "_P")
            .replace("/", "_S")
    )
}
