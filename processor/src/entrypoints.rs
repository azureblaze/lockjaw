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
use crate::parsing::FieldValue;
use crate::prologue::prologue_check;
use crate::type_data::ProcessorTypeData;
use crate::type_validator::TypeValidator;
use crate::{components, parsing, type_data};
use base64::engine::Engine;
use lazy_static::lazy_static;
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

    components::parse_provisions(&mut item_trait, &mut type_validator)?;

    let attributes = parsing::get_attribute_field_values(attr.clone())?;

    for key in attributes.keys() {
        if !ENTRY_POINT_METADATA_KEYS.contains(key) {
            return spanned_compile_error(attr.span(), &format!("unknown key: {}", key));
        }
    }
    let component_path = if let FieldValue::Path(span, path) =
        attributes.get("install_in").map_spanned_compile_error(
            attr.span(),
            "install_in metadata expected for #[entry_point]",
        )? {
        type_validator.add_dyn_path(path, span.clone());
        path
    } else {
        return spanned_compile_error(attr.span(), "path expected for install_in");
    };
    let entry_point_type_data =
        crate::type_data::from_local(&item_trait.ident.to_string(), item_trait.ident.span())?;
    let original_ident = item_trait.ident.clone();
    let original_vis = item_trait.vis.clone();
    let exported_ident = format_ident!("lockjaw_export_type_{}", original_ident);

    item_trait.ident = exported_ident.clone();
    item_trait.vis = Visibility::Public(Token![pub](item_trait.span()));

    let item_ident = item_trait.ident.clone();
    let prologue_check = prologue_check(item_trait.span());
    let validate_type = type_validator.validate(item_trait.ident.to_string());
    let getter_name = getter_name(
        &entry_point_type_data,
        &type_data::from_path_with_span(component_path, component_path.span())?,
    );
    let result = quote! {
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        #item_trait

        #original_vis use #exported_ident as #original_ident;

        #validate_type
        #prologue_check

        impl dyn #item_ident {
            fn get<'a>(component: &'a dyn #component_path) -> &'a dyn #item_ident {
                extern "Rust"{
                    fn #getter_name(component: &dyn #component_path) -> &'static dyn #item_ident;
                }
                unsafe { #getter_name(component) }
            }
        }
    };
    Ok(result)
}

pub fn getter_name(entry_point_type: &TypeData, component: &TypeData) -> Ident {
    format_ident!(
        "lockjaw_entry_point_getter_{}",
        base64::prelude::BASE64_STANDARD_NO_PAD
            .encode(format!(
                "{}_{}",
                entry_point_type.identifier().to_string(),
                component.identifier().to_string()
            ))
            .replace("+", "_P")
            .replace("/", "_S")
    )
}
