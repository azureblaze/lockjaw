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
use std::ops::Deref;

use crate::build_script_fatal;
use crate::manifest::BindingType::{Binds, BindsOptionOf, Multibinds, Provides};
use crate::manifest::{
    Binding, BindingType, Dependency, Manifest, Module, MultibindingMapKey, MultibindingType,
};
use crate::manifest_parser::Mod;
use crate::parsing;
use crate::parsing::{get_parenthesized_field_values, FieldValue};
use crate::type_data::TypeData;
use anyhow::Result;
use anyhow::{bail, Context};
use lazy_static::lazy_static;
use proc_macro2::TokenStream;
use std::convert::TryFrom;
use std::iter::FromIterator;
use syn::ImplItemFn;
use syn::__private::ToTokens;
use syn::spanned::Spanned;

lazy_static! {
    static ref MODULE_METADATA_KEYS: HashSet<String> = {
        let mut set = HashSet::<String>::new();
        set.insert("subcomponents".to_owned());
        set.insert("install_in".to_owned());
        set
    };
}

pub fn handle_module_attribute(
    attr: TokenStream,
    input: TokenStream,
    mod_: &Mod,
) -> Result<Manifest> {
    handle_module_attribute_internal(attr, input, mod_)
}

fn handle_module_attribute_internal(
    attr: TokenStream,
    input: TokenStream,
    mod_: &Mod,
) -> Result<Manifest> {
    let attributes = parsing::get_attribute_field_values(attr.clone())?;

    for key in attributes.keys() {
        if !MODULE_METADATA_KEYS.contains(key) {
            bail!("unknown key: {}", key);
        }
    }

    let module_path;
    let mut item_impl: syn::ItemImpl =
        syn::parse2(input.clone()).with_context(|| "impl expected")?;
    if let syn::Type::Path(path) = item_impl.self_ty.deref() {
        module_path = path.path.to_token_stream().to_string().replace(" ", "");
    } else {
        bail!("path expected");
    }
    let module_type = crate::type_data::from_local(&module_path.to_owned(), mod_)?;
    let mut bindings: Vec<Binding> = Vec::new();
    for i in 0..item_impl.items.len() {
        #[allow(unused_mut)] // required
        let mut item = item_impl.items.get_mut(i).unwrap();
        if let syn::ImplItem::Fn(ref mut method) = item {
            bindings.push(parse_binding(method, mod_)?);
        }
    }

    let mut module = Module::new();
    module.type_data = module_type;
    module.bindings.extend(bindings);
    if let Some(subcomponents) = attributes.get("subcomponents") {
        let types = subcomponents.get_types(mod_)?;
        module.subcomponents = HashSet::from_iter(types);
    }
    if let Some(install_in) = attributes.get("install_in") {
        let types = install_in.get_types(mod_)?;
        module.install_in = HashSet::from_iter(types);
    }
    let mut manifest = Manifest::new();

    manifest.modules.push(module);

    Ok(manifest)
}

fn parse_binding(method: &ImplItemFn, mod_: &Mod) -> Result<Binding> {
    let mut option_binding: Option<Binding> = None;
    let mut multibinding = MultibindingType::None;
    let mut map_key = MultibindingMapKey::None;
    let mut qualifier: Option<Box<TypeData>> = None;
    for attr in &method.attrs {
        let attr_str = parsing::get_attribute(attr);
        match attr_str.as_str() {
            "provides" => {
                if option_binding.is_some() {
                    bail!("#[module] methods can only be annotated by one of #[provides]/#[binds]/#[binds_option_of]/#[multibinds]");
                }
                option_binding = Some(handle_provides(attr, &method.sig, mod_)?);
            }
            "binds" => {
                if option_binding.is_some() {
                    bail!("#[module] methods can only be annotated by one of #[provides]/#[binds]/#[binds_option_of]/#[multibinds]");
                }
                option_binding = Some(handle_binds(attr, &method.sig, &method.block, mod_)?);
            }
            "binds_option_of" => {
                if option_binding.is_some() {
                    bail!("#[module] methods can only be annotated by one of #[provides]/#[binds]/#[binds_option_of]/#[multibinds]");
                }
                option_binding = Some(handle_binds_option_of(&method.sig, &method.block, mod_)?);
            }
            "multibinds" => {
                if option_binding.is_some() {
                    bail!("#[module] methods can only be annotated by one of #[provides]/#[binds]/#[binds_option_of]/#[multibinds]");
                }
                option_binding = Some(handle_multibinds(&method.sig, &method.block, mod_)?);
            }
            "into_vec" => {
                multibinding = MultibindingType::IntoVec;
            }
            "elements_into_vec" => {
                multibinding = MultibindingType::ElementsIntoVec;
                if let syn::ReturnType::Type(ref _token, ref ty) = method.sig.output {
                    let return_type = crate::type_data::from_syn_type(ty.deref(), mod_)?;
                    if return_type.path != "std::vec::Vec" {
                        build_script_fatal!(
                            method.span(),
                            mod_,
                            "#[elements_into_set] must return Vec<T>"
                        );
                    }
                }
            }
            "qualified" => {
                qualifier = Some(Box::new(parsing::get_type(
                    &attr.meta.require_list().unwrap().tokens,
                    mod_,
                )?));
            }
            "into_map" => {
                multibinding = MultibindingType::IntoMap;
                let fields = get_parenthesized_field_values(&attr.meta)?;
                if let Some(field) = fields.get("string_key") {
                    if let FieldValue::StringLiteral(ref string) = field {
                        map_key = MultibindingMapKey::String(string.clone());
                    } else {
                        bail!("string literal expected for string_key",);
                    }
                } else if let Some(field) = fields.get("i32_key") {
                    if let FieldValue::IntLiteral(ref int) = field {
                        map_key = MultibindingMapKey::I32(
                            i32::try_from(*int).with_context(|| "key overflows i32")?,
                        );
                    } else {
                        bail!("i32 literal expected for i32_key",);
                    }
                } else if let Some(field) = fields.get("enum_key") {
                    if let FieldValue::Path(ref path) = field {
                        let value_type = crate::type_data::from_path(path, mod_)?;
                        let mut enum_type = value_type.clone();
                        enum_type.path.truncate(
                            enum_type
                                .path
                                .rfind("::")
                                .with_context(|| "enum value should have at least one segment")?,
                        );
                        map_key = MultibindingMapKey::Enum(enum_type, value_type);
                    } else {
                        bail!("i32 literal expected for i32_key",);
                    }
                }
            }
            _ => {}
        }
    }
    if option_binding.is_none() {
        bail!("#[module] methods can only be annotated by #[provides]/#[binds]/#[binds_option_of]",);
    }
    let mut binding = option_binding.unwrap();
    if binding.binding_type == BindingType::Binds {
        if multibinding == MultibindingType::ElementsIntoVec {
            bail!("#[elements_into_set] cannot be used on #[binds]",);
        }
    }

    if multibinding == MultibindingType::ElementsIntoVec {
        if binding.type_data.path.ne("std::vec::Vec") {
            bail!("#[elements_into_set] must return Vec<T>");
        }
    }
    binding.multibinding_type = multibinding;
    binding.map_key = map_key;
    binding.type_data.qualifier = qualifier;
    Ok(binding)
}

fn handle_provides(
    attr: &syn::Attribute,
    signature: &syn::Signature,
    mod_: &Mod,
) -> Result<Binding> {
    let mut provides = Binding::new(Provides);
    provides.name = signature.ident.to_string();
    if let syn::ReturnType::Type(ref _token, ref ty) = signature.output {
        provides.type_data = crate::type_data::from_syn_type(ty.deref(), mod_)?;
    } else {
        bail!("return type expected");
    }
    for args in &signature.inputs {
        match args {
            syn::FnArg::Receiver(ref receiver) => {
                if receiver.reference.is_none() {
                    bail!("modules should not consume self");
                }
                provides.field_static = false;
            }
            syn::FnArg::Typed(ref type_) => {
                let mut dependency = Dependency::new();
                if let syn::Pat::Ident(ref ident) = type_.pat.deref() {
                    dependency.name = ident.ident.to_string()
                } else {
                    bail!("identifier expected");
                }
                dependency.type_data = crate::type_data::from_syn_type(type_.ty.deref(), mod_)?;
                provides.dependencies.push(dependency);
            }
        }
    }
    let provides_attr = parsing::get_parenthesized_field_values(&attr.meta)?;
    if let Some(scope) = provides_attr.get("scope") {
        let scopes = parsing::get_types(Some(scope), mod_)?;

        provides.type_data.scopes.extend(scopes);
    }
    Ok(provides)
}

fn handle_binds(
    attr: &syn::Attribute,
    signature: &syn::Signature,
    block: &syn::Block,
    mod_: &Mod,
) -> Result<Binding> {
    if !block.stmts.is_empty() {
        bail!("#[binds] methods must have empty body");
    }

    let mut binds = Binding::new(Binds);
    binds.name = signature.ident.to_string();
    if let syn::ReturnType::Type(ref _token, ref ty) = signature.output {
        let return_type = crate::type_data::from_syn_type(ty.deref(), mod_)?;
        match return_type.path.as_str() {
            "lockjaw::Cl" => {}
            "Cl" => {}
            _ => {
                build_script_fatal!(signature.span(), mod_, "#[binds] methods must return Cl<T>")
            }
        }
        binds.type_data = return_type.args[0].clone();
    } else {
        bail!("return type expected");
    }
    if signature.inputs.len() != 1 {
        bail!("binds method must only take the binding type as parameter",);
    }
    let args = signature.inputs.first().expect("missing binds arg");
    match args {
        syn::FnArg::Receiver(ref _receiver) => {
            bail!("binds method must only take the binding type as parameter",);
        }
        syn::FnArg::Typed(ref type_) => {
            let mut dependency = Dependency::new();
            if let syn::Pat::Ident(ref ident) = type_.pat.deref() {
                dependency.name = ident.ident.to_string();
            } else {
                bail!("identifier expected");
            }
            dependency.type_data = crate::type_data::from_syn_type(type_.ty.deref(), mod_)?;
            binds.dependencies.push(dependency);
        }
    }
    let provides_attr = parsing::get_parenthesized_field_values(&attr.meta)?;
    if let Some(scope) = provides_attr.get("scope") {
        let scopes = parsing::get_types(Some(scope), mod_)?;
        binds.type_data.scopes.extend(scopes);
    }
    Ok(binds)
}

fn handle_binds_option_of(
    signature: &syn::Signature,
    block: &syn::Block,
    mod_: &Mod,
) -> Result<Binding> {
    if !block.stmts.is_empty() {
        bail!("#[binds_option_of] methods must have empty body",);
    }
    let mut binds_option_of = Binding::new(BindsOptionOf);
    if let syn::ReturnType::Type(ref _token, ref ty) = signature.output {
        let return_type = crate::type_data::from_syn_type(ty.deref(), mod_)?;
        binds_option_of.type_data = return_type;
    } else {
        bail!("return type expected");
    }
    if signature.inputs.len() != 0 {
        bail!("binds_option_of method must only take no parameter",);
    }
    Ok(binds_option_of)
}

fn handle_multibinds(
    signature: &syn::Signature,
    block: &syn::Block,
    mod_: &Mod,
) -> Result<Binding> {
    if !block.stmts.is_empty() {
        bail!("#[multibinds] methods must have empty body");
    }
    if let syn::ReturnType::Type(ref _token, ref ty) = signature.output {
        let return_type = crate::type_data::from_syn_type(ty.deref(), mod_)?;
        match return_type.path.as_str() {
            "std::vec::Vec" => {}
            "std::collections::HashMap" => {}
            _ => {
                build_script_fatal!(
                    signature.span(),
                    mod_,
                    "#[multibinds] methods must return Vec<T> or HashMap<K,V>"
                );
            }
        }
    }
    let mut binds = Binding::new(Multibinds);
    binds.name = signature.ident.to_string();
    if let syn::ReturnType::Type(ref _token, ref ty) = signature.output {
        let return_type = crate::type_data::from_syn_type(ty.deref(), mod_)?;
        binds.type_data = return_type.clone();
    } else {
        bail!("return type expected");
    }
    if !signature.inputs.is_empty() {
        bail!("#[multibinds] method must take no arguments",);
    }
    Ok(binds)
}
