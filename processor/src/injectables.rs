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

use std::collections::{HashMap, HashSet};

use lazy_static::lazy_static;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{FnArg, ImplItem, ImplItemMethod, Pat};

use crate::error::{spanned_compile_error, CompileError};
use crate::manifest::{Dependency, Injectable};
use crate::parsing::FieldValue;
use crate::prologue::prologue_check;
use crate::type_data::TypeData;
use crate::type_validator::TypeValidator;
use crate::{manifest, parsing};

lazy_static! {
    static ref INJECTABLE_METADATA_KEYS: HashSet<String> = {
        let mut set = HashSet::<String>::new();
        set.insert("scope".to_owned());
        set.insert("container".to_owned());
        set
    };
}

lazy_static! {
    static ref FACTORY_METADATA_KEYS: HashSet<String> = {
        let mut set = HashSet::<String>::new();
        set.insert("implementing".to_owned());
        set
    };
}

#[derive(PartialEq)]
enum CtorType {
    Inject,
    Factory,
}

pub fn handle_injectable_attribute(
    attr: TokenStream,
    input: TokenStream,
) -> Result<TokenStream, TokenStream> {
    let span = input.span();
    let mut item: syn::ItemImpl =
        syn::parse2(input).map_spanned_compile_error(span, "impl block expected")?;
    let mut type_validator = TypeValidator::new();

    let attributes = parsing::get_attribute_field_values(attr.clone())?;
    for key in attributes.keys() {
        if !INJECTABLE_METADATA_KEYS.contains(key) {
            return spanned_compile_error(attr.span(), &format!("unknown key: {}", key));
        }
    }

    let (ctor_type, ctor, fields) = get_ctor(item.span(), &mut item.items)?;
    if ctor_type == CtorType::Factory {
        let factory = handle_factory(item.self_ty.clone(), ctor.clone(), fields.clone())?;
        for arg in ctor.sig.inputs.iter_mut() {
            if let FnArg::Receiver(ref receiver) = arg {
                return spanned_compile_error(receiver.span(), &format!("self not allowed"));
            }
            if let FnArg::Typed(ref mut type_) = arg {
                let mut new_attrs = Vec::new();
                for attr in &type_.attrs {
                    match parsing::get_attribute(attr).as_str() {
                        "qualified" | "runtime" => {}
                        _ => new_attrs.push(attr.clone()),
                    }
                }
                type_.attrs = new_attrs;
            }
        }
        return Ok(quote! {
            #item
            #factory
        });
    }
    let mut dependencies = Vec::<Dependency>::new();
    for arg in ctor.sig.inputs.iter_mut() {
        if let FnArg::Receiver(ref receiver) = arg {
            return spanned_compile_error(receiver.span(), &format!("self not allowed"));
        }
        if let FnArg::Typed(ref mut type_) = arg {
            if let Pat::Ident(ref ident) = *type_.pat {
                let mut dependency = Dependency::new();
                dependency.type_data = TypeData::from_syn_type(&type_.ty)?;
                let mut new_attrs = Vec::new();
                for attr in &type_.attrs {
                    match parsing::get_attribute(attr).as_str() {
                        "qualified" => {
                            dependency.type_data.qualifier =
                                Some(Box::new(parsing::get_parenthesized_type(&attr.tokens)?))
                        }
                        _ => new_attrs.push(attr.clone()),
                    }
                }
                type_.attrs = Vec::new(); //new_attrs;
                dependency.name = ident.ident.to_string();
                dependencies.push(dependency);
            } else {
                return spanned_compile_error(type_.span(), &format!("identifier expected"));
            }
        }
    }
    let type_name;
    if let syn::Type::Path(ref path) = *item.self_ty {
        let segments: Vec<String> = path
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect();
        type_name = segments.join("::");
    } else {
        return spanned_compile_error(item.self_ty.span(), &format!("path expected"));
    }

    let mut injectable = Injectable::new();
    injectable.type_data = TypeData::from_local(&type_name, item.self_ty.span())?;
    let scopes = parsing::get_types(attributes.get("scope"), item.self_ty.span())?;
    for scope in &scopes {
        type_validator.add_type(scope, attr.span())
    }
    injectable.container = get_container(attr.span(), &attributes, &scopes)?;
    injectable.type_data.scopes.extend(scopes);
    injectable.ctor_name = ctor.sig.ident.to_string();
    injectable.dependencies.extend(dependencies);
    let identifier = injectable.type_data.identifier().to_string();

    manifest::with_manifest(|mut manifest| manifest.injectables.push(injectable));

    let type_check = type_validator.validate(identifier);
    let prologue_check = prologue_check(item.span());
    Ok(quote! {
        #item
        #type_check
        #prologue_check
    })
}

fn get_ctor(
    span: Span,
    items: &mut Vec<ImplItem>,
) -> Result<(CtorType, &mut ImplItemMethod, HashMap<String, FieldValue>), TokenStream> {
    let mut ctors = 0;
    for item in &mut *items {
        if let ImplItem::Method(ref mut method) = item {
            if parsing::has_attribute(&method.attrs, "inject")
                || parsing::has_attribute(&method.attrs, "factory")
            {
                ctors += 1;
                if ctors == 2 {
                    return spanned_compile_error(
                        item.span(),
                        "only one method can be marked with #[inject]/#[factory]",
                    );
                }
            }
        }
    }
    if ctors == 0 {
        return spanned_compile_error(
            span,
            "must have one method marked with #[inject]/#[factory]",
        );
    }
    for item in items {
        if let ImplItem::Method(ref mut method) = item {
            if parsing::has_attribute(&method.attrs, "inject") {
                let index = method
                    .attrs
                    .iter()
                    .position(|a| parsing::is_attribute(a, "inject"))
                    .unwrap();
                let fields =
                    parsing::get_parenthesized_field_values(method.attrs[index].tokens.clone())?;
                method.attrs.remove(index);
                return Ok((CtorType::Inject, method, fields));
            }
            if parsing::has_attribute(&method.attrs, "factory") {
                let index = method
                    .attrs
                    .iter()
                    .position(|a| parsing::is_attribute(a, "factory"))
                    .unwrap();
                let fields =
                    parsing::get_parenthesized_field_values(method.attrs[index].tokens.clone())?;
                method.attrs.remove(index);
                return Ok((CtorType::Factory, method, fields));
            }
        }
    }
    panic!("should have ctor")
}

fn get_container(
    span: Span,
    attributes: &HashMap<String, FieldValue>,
    scopes: &Vec<TypeData>,
) -> Result<Option<TypeData>, TokenStream> {
    if attributes.contains_key("container") {
        if let FieldValue::Path(span, path) = attributes.get("container").unwrap() {
            if scopes.is_empty() {
                return spanned_compile_error(
                    span.clone(),
                    "the 'container' metadata should only be used with an injectable that also has 'scope'",
                );
            }
            return Ok(Some(TypeData::from_path_with_span(path, span.clone())?));
        } else {
            return spanned_compile_error(span, "path expected for 'container'");
        }
    }
    Ok(None)
}

fn handle_factory(
    self_ty: Box<syn::Type>,
    method: ImplItemMethod,
    metadata: HashMap<String, FieldValue>,
) -> Result<TokenStream, TokenStream> {
    for (k, v) in &metadata {
        if !FACTORY_METADATA_KEYS.contains(k) {
            return spanned_compile_error(v.span(), &format!("unknown key: {}", k));
        }
    }
    let mut fields = quote! {};
    let mut fields_arg = quote! {};
    let mut runtime_args = quote! {};
    let mut args = quote! {};

    for arg in method.sig.inputs.iter() {
        if let FnArg::Receiver(ref receiver) = arg {
            return spanned_compile_error(receiver.span(), &format!("self not allowed"));
        }
        if let FnArg::Typed(ref type_) = arg {
            if let Pat::Ident(ref ident) = *type_.pat {
                if parsing::has_attribute(&type_.attrs, "runtime") {
                    let mut type_arg = type_.clone();
                    type_arg.attrs = Vec::new();
                    runtime_args = quote! {
                        #runtime_args
                        #type_arg,
                    };
                    args = quote! {
                        #args
                        #ident,
                    }
                } else {
                    let ty = &type_.ty;
                    fields = quote! {
                        #fields
                        #ident : ::lockjaw::Provider<'a, #ty>,
                    };
                    fields_arg = quote! {
                        #fields_arg
                        #ident,
                    };
                    args = quote! {
                        #args
                        self.#ident.get(),
                    }
                }
            } else {
                return spanned_compile_error(type_.span(), &format!("identifier expected"));
            }
        }
    }
    let mut factory_ty = self_ty.clone();
    if let syn::Type::Path(ref path) = self_ty.as_ref() {
        let ident = format_ident!("{}Factory", path.path.segments.last().unwrap().ident);
        if let syn::Type::Path(ref mut factory_path) = factory_ty.as_mut() {
            factory_path.path.segments.last_mut().unwrap().ident = ident;
        }
    } else {
        return spanned_compile_error(self_ty.span(), &format!("path expected"));
    }
    let method_name = method.sig.ident;
    let viz;
    let impl_for = if let Some(implementing) = metadata.get("implementing") {
        let trait_ = if let FieldValue::Path(_, path) = implementing {
            viz = quote! {};
            quote! {#path}
        } else {
            return spanned_compile_error(implementing.span(), "path expected for 'implementing'");
        };
        quote! {
            #trait_ for
        }
    } else {
        viz = quote! {pub};
        quote! {}
    };
    let result = quote! {
        pub struct #factory_ty<'a> {
            #fields
            lockjaw_phamtom_data: ::std::marker::PhantomData<&'a ::std::string::String>
        }
        #[injectable]
        impl <'a> #factory_ty<'a> {
            #[inject]
            fn lockjaw_new_factory(#fields) -> Self{
                Self{
                    #fields_arg
                    lockjaw_phamtom_data: ::std::marker::PhantomData
                }
            }
        }

        impl <'a> #impl_for #factory_ty<'a> {
            #viz fn #method_name(&self,#runtime_args) -> #self_ty {
                #self_ty::#method_name(#args)
            }
        }
    };

    // log!("{}", result.to_string());
    Ok(result)
}
