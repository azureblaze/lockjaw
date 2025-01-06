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

use crate::manifest::Manifest;
use crate::parsing::FieldValue;

use crate::attributes::components;
use crate::environment::current_crate;
use crate::manifest::{EntryPoint, ExpandedVisibility, TypeRoot};
use crate::manifest_parser::Mod;
use crate::type_data::TypeData;
use crate::{parsing, type_data};
use anyhow::{bail, Context, Result};
use lazy_static::lazy_static;
use proc_macro2::TokenStream;
use std::collections::HashSet;

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
    mod_: &Mod,
) -> Result<Manifest> {
    let item_trait: syn::ItemTrait = syn::parse2(input).with_context(|| "trait expected")?;

    let provisions = components::get_provisions(&item_trait, mod_)?;

    let attributes = parsing::get_attribute_field_values(attr.clone())?;

    for key in attributes.keys() {
        if !ENTRY_POINT_METADATA_KEYS.contains(key) {
            bail!("unknown key: {}", key);
        }
    }
    let component = if let FieldValue::Path(path) = attributes
        .get("install_in")
        .with_context(|| "install_in metadata expected for #[entry_point]")?
    {
        type_data::from_path(path, mod_)?
    } else {
        bail!("path expected for install_in");
    };
    let mut entry_point = EntryPoint::new();
    entry_point.type_data = type_data::from_local(&item_trait.ident.to_string(), mod_)?;

    entry_point.provisions.extend(provisions);
    entry_point.component = component.clone();

    let original_ident = item_trait.ident.to_string();
    let exported_ident = format!("lockjaw_export_type_{}", original_ident);

    let type_ = type_data::from_local(&original_ident, mod_)?;
    let crate_type = type_data::from_local(&exported_ident.to_string(), mod_)?;

    let mut manifest = Manifest::new();

    let mut exported_type = TypeData::new();
    exported_type.root = TypeRoot::CRATE;
    exported_type.path = type_.identifier_string();
    exported_type.field_crate = current_crate();

    manifest.expanded_visibilities.insert(
        type_.canonical_string_path_without_args(),
        ExpandedVisibility {
            crate_local_name: crate_type,
            exported_name: exported_type,
        },
    );

    manifest.entry_points.push(entry_point);

    Ok(manifest)
}
