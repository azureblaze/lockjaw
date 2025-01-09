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

use std::borrow::Borrow;
use std::collections::HashSet;
use std::ops::Deref;

use crate::environment::current_package;
use crate::manifest::{
    BuilderModules, Component, ComponentType, Dependency, ExpandedVisibility, Manifest, Module,
    TypeRoot,
};
use crate::manifest_parser::Mod;
use crate::parsing::FieldValue;
use crate::type_data;
use crate::type_data::{from_local, from_path, TypeData};
use crate::{build_script_fatal, parsing};
use anyhow::{bail, Context, Result};
use lazy_static::lazy_static;
use proc_macro2::TokenStream;
use syn::__private::ToTokens;
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
    definition_only: bool,
    mod_: &Mod,
) -> Result<Manifest> {
    let mut item_trait: ItemTrait = syn::parse2(input).with_context(|| "trait expected")?;

    let provisions = get_provisions(&mut item_trait, mod_)?;

    let attributes = parsing::get_attribute_field_values(attr.clone())?;
    for key in attributes.keys() {
        if !COMPONENT_METADATA_KEYS.contains(key) {
            if component_type == ComponentType::Subcomponent
                && SUBCOMPONENT_METADATA_KEYS.contains(key)
            {
                continue;
            }
            bail!("unknown key: {}", key);
        }
    }

    let builder_modules = if let Some(value) = attributes.get("builder_modules") {
        if let FieldValue::Path(ref path) = value {
            let type_ = type_data::from_path(path, mod_)?;
            Some(type_)
        } else {
            bail!("path expected for modules");
        }
    } else {
        None
    };

    let modules = if let Some(value) = attributes.get("modules") {
        match value {
            FieldValue::Path(ref path) => {
                let type_ = type_data::from_path(&path, mod_)?;
                Some(vec![type_])
            }
            FieldValue::Array(ref array) => {
                let mut result = Vec::new();
                for field in array {
                    if let FieldValue::Path(ref path) = field {
                        let type_ = type_data::from_path(&path, mod_)?;
                        result.push(type_)
                    } else {
                        bail!("path expected for modules");
                    }
                }
                Some(result)
            }
            _ => {
                bail!("path expected for modules");
            }
        }
    } else {
        None
    };

    let mut component = Component::new();
    component.name = item_trait.ident.to_string();
    component.type_data = type_data::from_local(&item_trait.ident.to_string(), mod_)?;
    component.component_type = component_type;
    component.provisions.extend(provisions);
    if let Some(ref m) = builder_modules {
        component.builder_modules = Some(m.clone());
    }
    if let Some(ref m) = modules {
        component.modules = m.clone();
    }
    component.definition_only = definition_only;
    component.address = from_local(
        &format!(
            "LOCKJAW_COMPONENT_BUILDER_ADDR_{}",
            &item_trait.ident.to_string()
        ),
        mod_,
    )?;
    let mut result = Manifest::new();
    if component.component_type == ComponentType::Component {
        let mut exported_addr_type = TypeData::new();
        exported_addr_type.root = TypeRoot::CRATE;
        exported_addr_type.path = component.address.identifier_string();
        exported_addr_type.field_crate = current_package();
        result.expanded_visibilities.insert(
            component.address.canonical_string_path(),
            ExpandedVisibility {
                crate_local_name: component.address.clone(),
                exported_name: exported_addr_type,
            },
        );
    }

    if let Some(parent) = attributes.get("parent") {
        if let FieldValue::Path(path) = parent {
            let module_name = format!("lockjaw_parent_module_{}", item_trait.ident.to_string());
            let subcomponent_name = item_trait.ident.to_string();
            result.modules.push(Module {
                type_data: from_local(&module_name, mod_)?,
                bindings: vec![],
                subcomponents: HashSet::from([from_local(&subcomponent_name, mod_)?]),
                install_in: HashSet::from([from_path(path, mod_)?]),
            });
        } else {
            bail!("path expected for parent");
        }
    };
    result.components.push(component);
    Ok(result)
}

pub fn get_provisions(item_trait: &ItemTrait, mod_: &Mod) -> Result<Vec<Dependency>> {
    let mut provisions = Vec::<Dependency>::new();
    for item in &item_trait.items {
        if let syn::TraitItem::Fn(ref method) = item {
            let mut provision = Dependency::new();
            let mut qualifier: Option<TypeData> = None;
            let mut new_attrs: Vec<Attribute> = Vec::new();
            for attr in &method.attrs {
                match parsing::get_attribute(attr).as_str() {
                    "qualified" => {
                        qualifier = Some(parsing::get_type(
                            &attr.meta.require_list().unwrap().tokens,
                            mod_,
                        )?);
                    }
                    _ => new_attrs.push(attr.clone()),
                }
            }
            provision.name = method.sig.ident.to_string();
            if let syn::ReturnType::Type(ref _token, ref ty) = method.sig.output {
                if is_trait_object_without_lifetime(ty.deref(), mod_)? {
                    let path = type_data::from_local(&item_trait.ident.to_string(), mod_)?;
                    build_script_fatal!("in {}::{}:\ntrait object return type may depend on scoped objects, and must have lifetime bounded by the component", path.canonical_string_path(), method.sig.ident);
                }
                provision.type_data = type_data::from_syn_type(ty.deref(), mod_)?;
                provision.type_data.qualifier = qualifier.map(Box::new);
            } else {
                bail!("return type expected for component provisions",);
            }
            provisions.push(provision);
        }
    }
    Ok(provisions)
}

fn is_trait_object_without_lifetime(ty: &syn::Type, mod_: &Mod) -> Result<bool> {
    let type_ = type_data::from_syn_type(ty, mod_)?;
    if type_.root == TypeRoot::GLOBAL && type_.path == "lockjaw::Cl" {
        return Ok(false);
    }
    let tokens: Vec<String> = ty
        .to_token_stream()
        .into_iter()
        .map(|t| t.to_string())
        .collect();
    if !tokens.contains(&"dyn".to_owned()) {
        return Ok(false);
    }
    Ok(!tokens.contains(&"'".to_owned()))
}

pub fn handle_builder_modules_attribute(
    _attr: TokenStream,
    input: TokenStream,
    mod_: &Mod,
) -> Result<Manifest> {
    let item_struct: syn::ItemStruct = syn::parse2(input).with_context(|| "struct expected")?;
    let mut modules = <Vec<Dependency>>::new();

    for field in &item_struct.fields {
        let mut dep = Dependency::new();
        let name = field
            .ident
            .as_ref()
            .with_context(|| "#[builder_modules] cannot be tuples")?;
        dep.name = name.to_string();
        dep.type_data = type_data::from_syn_type(field.ty.borrow(), mod_)?;
        modules.push(dep);
    }

    let mut builder_modules = BuilderModules::new();
    builder_modules.type_data = Some(type_data::from_local(&item_struct.ident.to_string(), mod_)?);
    builder_modules.builder_modules.extend(modules);
    let mut result = Manifest::new();
    result.builder_modules.push(builder_modules);

    Ok(result)
}
