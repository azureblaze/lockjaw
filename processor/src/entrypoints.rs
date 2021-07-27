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
use crate::manifest::{with_manifest, EntryPoint};
use crate::parsing::FieldValue;
use crate::prologue::prologue_check;
use crate::type_data::TypeData;
use crate::type_validator::TypeValidator;
use crate::{components, parsing};
use lazy_static::lazy_static;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::collections::HashSet;
use syn::spanned::Spanned;

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
        TypeData::from_path_with_span(path, span.clone())?
    } else {
        return spanned_compile_error(attr.span(), "path expected for install_in");
    };

    let mut entry_point = EntryPoint::new();
    entry_point.type_data =
        TypeData::from_local(&item_trait.ident.to_string(), item_trait.ident.span())?;

    entry_point.provisions.extend(provisions);
    entry_point.component = component.clone();

    with_manifest(|mut manifest| manifest.entry_points.push(entry_point.clone()));

    let identifier = entry_point.type_data.identifier().to_string();
    let item_ident = item_trait.ident.clone();
    let component_type = component.syn_type();
    let prologue_check = prologue_check(item_trait.span());
    let validate_type = type_validator.validate(identifier);
    let getter_name = getter_name(&entry_point);
    let result = quote! {
        #item_trait
        #validate_type
        #prologue_check

        impl dyn #item_ident {
            fn get<'a>(component: &'a dyn #component_type) -> &'a dyn #item_ident {
                extern "Rust"{
                    pub fn #getter_name(component: &dyn #component_type) -> &'static dyn #item_ident;
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
        base64::encode_config(
            format!(
                "{}_{}",
                entry_point.type_data.identifier().to_string(),
                entry_point.component.identifier().to_string()
            ),
            base64::Config::new(base64::CharacterSet::Standard, false)
        )
        .replace("+", "_P")
        .replace("/", "_S")
    )
}
