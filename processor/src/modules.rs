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

use crate::error::{spanned_compile_error, CompileError};
use crate::manifests::type_from_syn_type;
use crate::protos::manifest::{Dependency, Module, Provider, Type, Type_Root};
use crate::{environment, manifests, parsing};
use lazy_static::lazy_static;
use proc_macro2::TokenStream;
use quote::quote;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Deref;
use syn::export::ToTokens;
use syn::spanned::Spanned;

thread_local! {
    static MODULES :RefCell<HashMap<String, LocalModule>> = RefCell::new(HashMap::new());
}

lazy_static! {
    static ref MODULE_METADATA_KEYS: HashSet<String> = {
        let mut set = HashSet::<String>::new();
        set.insert("path".to_owned());
        set
    };
}

lazy_static! {
    static ref MODULE_IMPL_METADATA_KEYS: HashSet<String> = {
        let mut set = HashSet::<String>::new();
        set.insert("path".to_owned());
        set
    };
}

struct LocalModule {
    name: String,
    providers: Vec<Provider>,
    additional_path: Option<String>,
}

pub fn handle_module_attribute(
    attr: TokenStream,
    input: TokenStream,
) -> Result<TokenStream, TokenStream> {
    let span = input.span();
    let item_struct: syn::ItemStruct =
        syn::parse2(input.clone()).map_spanned_compile_error(span, "struct expected")?;
    let attributes = parsing::get_attribute_metadata(attr.clone())?;

    for key in attributes.keys() {
        if !MODULE_METADATA_KEYS.contains(key) {
            return spanned_compile_error(attr.span(), &format!("unknown key: {}", key));
        }
    }

    let module = LocalModule {
        name: item_struct.ident.to_string(),
        additional_path: attributes.get("path").cloned(),
        providers: Vec::new(),
    };

    MODULES.with(|module_map| {
        module_map.borrow_mut().insert(
            with_additional_path(&item_struct.ident.to_string(), attributes.get("path")),
            module,
        )
    });

    Ok(input)
}

pub fn handle_module_impl_attribute(
    attr: TokenStream,
    input: TokenStream,
) -> Result<TokenStream, TokenStream> {
    MODULES.with(|mm| {
        let span = input.span();
        let attributes = parsing::get_attribute_metadata(attr.clone())?;

        for key in attributes.keys() {
            if !MODULE_IMPL_METADATA_KEYS.contains(key) {
                return spanned_compile_error(attr.span(), &format!("unknown key: {}", key));
            }
        }

        let mut module_map = mm.borrow_mut();
        let mut item_impl: syn::ItemImpl =
            syn::parse2(input.clone()).map_spanned_compile_error(span, "impl expected")?;
        let module: &mut LocalModule;
        if let syn::Type::Path(path) = item_impl.self_ty.deref() {
            module = module_map
                .get_mut(&with_additional_path(
                    &path.path.to_token_stream().to_string().replace(" ", ""),
                    attributes.get("path"),
                ))
                .map_spanned_compile_error(
                    path.path.span(),
                    "module not registered. add #[module] to the struct first",
                )?;
        } else {
            return spanned_compile_error(item_impl.span(), "path expected");
        }
        let mut removed_item_indices = Vec::<usize>::new();

        for i in 0..item_impl.items.len() {
            #[allow(unused_mut)] // required
            let mut item = item_impl.items.get_mut(i).unwrap();
            if let syn::ImplItem::Method(ref mut method) = item {
                let mut new_attrs: Vec<syn::Attribute> = Vec::new();
                for attr in &method.attrs {
                    if parsing::is_attribute(attr, "provides") {
                        let mut proto_provider = Provider::new();
                        proto_provider.set_name(method.sig.ident.to_string());
                        if let syn::ReturnType::Type(ref _token, ref ty) = method.sig.output {
                            proto_provider.set_field_type(type_from_syn_type(ty.deref())?);
                        } else {
                            return spanned_compile_error(
                                method.sig.span(),
                                "return type expected",
                            );
                        }
                        for args in &method.sig.inputs {
                            match args {
                                syn::FnArg::Receiver(ref receiver) => {
                                    if receiver.reference.is_none() {
                                        return spanned_compile_error(
                                            args.span(),
                                            "modules should not consume self",
                                        );
                                    }
                                    proto_provider.set_field_static(false);
                                }
                                syn::FnArg::Typed(ref type_) => {
                                    let mut dependency = Dependency::new();
                                    if let syn::Pat::Ident(ref ident) = type_.pat.deref() {
                                        dependency.set_name(ident.ident.to_string())
                                    } else {
                                        return spanned_compile_error(
                                            args.span(),
                                            "identifier expected",
                                        );
                                    }
                                    dependency
                                        .set_field_type(type_from_syn_type(type_.ty.deref())?);
                                    proto_provider.mut_dependencies().push(dependency);
                                }
                            }
                        }
                        let provides_attr =
                            parsing::get_parenthesized_attribute_metadata(attr.tokens.clone())?;
                        let scopes =
                            parsing::get_types(provides_attr.get("scope").map(Clone::clone))?;
                        manifests::extend(proto_provider.mut_field_type().mut_scopes(), scopes);
                        module.providers.push(proto_provider);
                    } else if parsing::is_attribute(attr, "binds") {
                        let mut proto_provider = Provider::new();
                        proto_provider.set_binds(true);
                        proto_provider.set_name(method.sig.ident.to_string());
                        if let syn::ReturnType::Type(ref _token, ref ty) = method.sig.output {
                            proto_provider.set_field_type(type_from_syn_type(ty.deref())?);
                        } else {
                            return spanned_compile_error(
                                method.sig.span(),
                                "return type expected",
                            );
                        }
                        if method.sig.inputs.len() != 1 {
                            return spanned_compile_error(
                                method.sig.span(),
                                "binds method must only take the binding type as parameter",
                            );
                        }
                        let args = method.sig.inputs.first().expect("missing binds arg");
                        match args {
                            syn::FnArg::Receiver(ref _receiver) => {
                                return spanned_compile_error(
                                    args.span(),
                                    "binds method must only take the binding type as parameter",
                                );
                            }
                            syn::FnArg::Typed(ref type_) => {
                                let mut dependency = Dependency::new();
                                if let syn::Pat::Ident(ref ident) = type_.pat.deref() {
                                    dependency.set_name(ident.ident.to_string());
                                } else {
                                    return spanned_compile_error(
                                        args.span(),
                                        "identifier expected",
                                    );
                                }
                                dependency.set_field_type(type_from_syn_type(type_.ty.deref())?);
                                proto_provider.mut_dependencies().push(dependency);
                            }
                        }
                        let provides_attr =
                            parsing::get_parenthesized_attribute_metadata(attr.tokens.clone())?;
                        let scopes =
                            parsing::get_types(provides_attr.get("scope").map(Clone::clone))?;
                        manifests::extend(proto_provider.mut_field_type().mut_scopes(), scopes);
                        module.providers.push(proto_provider);
                        removed_item_indices.push(i);
                    } else {
                        new_attrs.push(attr.clone());
                    }
                }
                method.attrs = new_attrs;
            }
        }
        removed_item_indices.reverse();
        for i in removed_item_indices {
            item_impl.items.remove(i);
        }

        Ok(quote! {#item_impl})
    })
}

fn with_additional_path(path: &str, additional_path: Option<&String>) -> String {
    format!(
        "{}{}{}",
        additional_path.unwrap_or(&"".to_owned()),
        if additional_path.is_some() { "::" } else { "" },
        path
    )
}

pub fn generate_manifest(base_path: &str) -> Vec<Module> {
    MODULES.with(|m| {
        let mut modules = m.borrow_mut();
        let mut result = Vec::<Module>::new();
        for local_module in modules.values() {
            let mut module = Module::new();
            let mut type_ = Type::new();
            type_.set_field_crate(environment::current_crate());
            type_.set_root(Type_Root::CRATE);
            let mut path = String::new();
            if !base_path.is_empty() {
                path.push_str(base_path);
                path.push_str("::");
            }
            if let Some(additional_path) = &local_module.additional_path {
                path.push_str(additional_path);
                path.push_str("::");
            }
            path.push_str(&local_module.name);

            type_.set_path(path);
            module.set_field_type(type_);
            manifests::extend(module.mut_providers(), local_module.providers.clone());
            result.push(module);
        }
        modules.clear();
        result
    })
}
