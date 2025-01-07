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

use crate::error::{spanned_compile_error, CompileError};
use crate::parsing;
use crate::parsing::FieldValue;

use crate::type_validator::TypeValidator;
use lazy_static::lazy_static;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{FnArg, ImplItem, ImplItemFn, Pat, PathArguments, Visibility};

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
        set.insert("visibility".to_owned());
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
    for arg in ctor.sig.inputs.iter_mut() {
        if let FnArg::Receiver(ref receiver) = arg {
            return spanned_compile_error(receiver.span(), &format!("self not allowed"));
        }
        if let FnArg::Typed(ref mut type_) = arg {
            if let Pat::Ident(_) = *type_.pat {
                let mut new_attrs = Vec::new();
                for attr in &type_.attrs {
                    match parsing::get_attribute(attr).as_str() {
                        "qualified" => {
                            type_validator.add_path(
                                &parsing::get_path(&attr.meta.require_list().unwrap().tokens)?,
                                attr.span(),
                            );
                        }
                        _ => new_attrs.push(attr.clone()),
                    }
                }
                type_.attrs = Vec::new(); //new_attrs;
            } else {
                return spanned_compile_error(type_.span(), &"identifier expected".to_string());
            }
        }
    }

    if let Some(scopes) = attributes.get("scope") {
        for (path, span) in scopes.get_paths()? {
            type_validator.add_dyn_path(&path, span);
        }
    } else {
        if attributes.contains_key("container") {
            return spanned_compile_error(
                    span.clone(),
                    "the 'container' metadata should only be used with an injectable that also has 'scope'",
                );
        }
    }
    validate_container(attr.span(), &attributes, &mut type_validator, &item.self_ty)?;

    let type_check = type_validator.validate(parsing::type_string(&item.self_ty)?);

    let result = quote! {
        #item
        #type_check
    };
    //log!("{}", result.to_string());
    Ok(result)
}

fn get_ctor(
    span: Span,
    items: &mut Vec<ImplItem>,
) -> Result<(CtorType, &mut ImplItemFn, HashMap<String, FieldValue>), TokenStream> {
    let mut ctors = 0;
    for item in &mut *items {
        if let ImplItem::Fn(ref mut method) = item {
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
        if let ImplItem::Fn(ref mut method) = item {
            if parsing::has_attribute(&method.attrs, "inject") {
                let index = method
                    .attrs
                    .iter()
                    .position(|a| parsing::is_attribute(a, "inject"))
                    .unwrap();
                let fields = parsing::get_parenthesized_field_values(&method.attrs[index].meta)?;
                method.attrs.remove(index);
                return Ok((CtorType::Inject, method, fields));
            }
            if parsing::has_attribute(&method.attrs, "factory") {
                let index = method
                    .attrs
                    .iter()
                    .position(|a| parsing::is_attribute(a, "factory"))
                    .unwrap();
                let fields = parsing::get_parenthesized_field_values(&method.attrs[index].meta)?;
                method.attrs.remove(index);
                return Ok((CtorType::Factory, method, fields));
            }
        }
    }
    panic!("should have ctor")
}

fn validate_container(
    span: Span,
    attributes: &HashMap<String, FieldValue>,
    type_validator: &mut TypeValidator,
    element_type: &syn::Type,
) -> Result<(), TokenStream> {
    if attributes.contains_key("container") {
        if let FieldValue::Path(span, path) = attributes.get("container").unwrap() {
            type_validator.add_path_and_arg(path, span.clone(), element_type);
        } else {
            return spanned_compile_error(span, "path expected for 'container'");
        }
    }
    Ok(())
}

fn handle_factory(
    mut self_ty: Box<syn::Type>,
    method: ImplItemFn,
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
    let mut lifetime = quote! {};
    let mut factory_ty = self_ty.clone();
    if let syn::Type::Path(ref mut path) = self_ty.as_mut() {
        let last_segment = path.path.segments.last_mut().unwrap();
        if last_segment.arguments != PathArguments::None {
            lifetime = quote! {<'a>};
            last_segment.arguments = PathArguments::None;
        }

        let ident = format_ident!("{}Factory", path.path.segments.last().unwrap().ident);
        if let syn::Type::Path(ref mut factory_path) = factory_ty.as_mut() {
            let last_segment = factory_path.path.segments.last_mut().unwrap();
            last_segment.ident = ident;
            last_segment.arguments = PathArguments::None;
        }
    } else {
        return spanned_compile_error(self_ty.span(), &format!("path expected"));
    }
    let method_name = method.sig.ident;
    let method_viz;
    let impl_for = if let Some(implementing) = metadata.get("implementing") {
        let trait_ = if let FieldValue::Path(_, path) = implementing {
            method_viz = quote! {};
            quote! {#path}
        } else {
            return spanned_compile_error(implementing.span(), "path expected for 'implementing'");
        };
        quote! {
            #trait_ for
        }
    } else {
        method_viz = quote! {pub};
        quote! {}
    };
    let component_visible;
    let factory_viz = if let Some(visibility) = metadata.get("visibility") {
        if let FieldValue::StringLiteral(span, vis_string) = visibility {
            let syn_visibility: Visibility = syn::parse_str(vis_string).map_spanned_compile_error(
                span.clone(),
                "visibility specifier string('pub', 'pub(crate)', 'pub(in some::path)') expected",
            )?;
            if let Visibility::Public(_) = syn_visibility {
                component_visible = quote! {};
            } else {
                component_visible = quote! {#[::lockjaw::component_visible]};
            }
            quote! {#syn_visibility}
        } else {
            return spanned_compile_error(visibility.span(), "string expected for `visibility`");
        }
    } else {
        component_visible = quote! {#[::lockjaw::component_visible]};
        quote! {}
    };

    let result = quote! {
        #component_visible
        #factory_viz struct #factory_ty<'a> {
            #fields
            lockjaw_phamtom_data: ::std::marker::PhantomData<&'a ::std::string::String>
        }
        #[::lockjaw::injectable]
        impl <'a> #factory_ty<'a> {
            #[doc(hidden)]
            #[inject]
            pub fn lockjaw_new_factory(#fields) -> Self{
                Self{
                    #fields_arg
                    lockjaw_phamtom_data: ::std::marker::PhantomData
                }
            }
        }

        impl <'a> #impl_for #factory_ty<'a> {
            #method_viz fn #method_name(&self,#runtime_args) -> #self_ty #lifetime {
                #self_ty::#method_name(#args)
            }
        }
    };

    //log!("{}", result.to_string());
    Ok(result)
}
