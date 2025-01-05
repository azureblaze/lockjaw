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

use crate::manifest::{ExpandedVisibility, Manifest, TypeRoot};
use crate::type_data;
use crate::type_data::TypeData;

use crate::manifest_parser::Mod;
use anyhow::{bail, Result};
use proc_macro2::TokenStream;
use syn::{ItemStruct, ItemTrait};

pub fn handle_component_visible_attribute(
    _attr: TokenStream,
    input: TokenStream,
    _mod: &Mod,
) -> Result<Manifest> {
    if let Ok(item_struct) = syn::parse2::<syn::ItemStruct>(input.clone()) {
        return handle_item_struct(item_struct, _mod);
    };

    if let Ok(item_trait) = syn::parse2::<syn::ItemTrait>(input.clone()) {
        return handle_item_trait(item_trait, _mod);
    };
    bail!("unable to handle the item")
}

fn handle_item_struct(item_struct: ItemStruct, mod_: &Mod) -> Result<Manifest> {
    let original_ident = item_struct.ident.clone();
    let exported_ident = format!("lockjaw_export_type_{}", original_ident);

    let type_ = type_data::from_local(&original_ident.to_string(), mod_)?;
    let crate_type = type_data::from_local(&exported_ident, mod_)?;

    let mut manifest = Manifest::new();
    let mut exported_type = TypeData::new();
    exported_type.root = TypeRoot::CRATE;
    exported_type.path = type_.identifier_string();
    exported_type.field_crate = crate::environment::current_crate();

    manifest.expanded_visibilities.insert(
        type_.canonical_string_path(),
        ExpandedVisibility {
            crate_local_name: crate_type,
            exported_name: exported_type,
        },
    );

    Ok(manifest)
}

fn handle_item_trait(item_trait: ItemTrait, mod_: &Mod) -> Result<Manifest> {
    let original_ident = item_trait.ident.to_string();
    let exported_ident = format!("lockjaw_export_type_{}", original_ident);

    let mut type_ = type_data::from_local(&original_ident, mod_)?;
    type_.trait_object = true;
    let crate_type = type_data::from_local(&exported_ident, mod_)?;

    let mut manifest = Manifest::new();
    let mut exported_type = TypeData::new();
    exported_type.root = TypeRoot::CRATE;
    exported_type.path = type_.identifier_string();
    exported_type.field_crate = crate::environment::current_crate();
    exported_type.trait_object = true;

    manifest.expanded_visibilities.insert(
        type_.canonical_string_path(),
        ExpandedVisibility {
            crate_local_name: crate_type,
            exported_name: exported_type,
        },
    );
    Ok(manifest)
}
