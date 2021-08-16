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

use crate::type_data::TypeData;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};

pub struct TypeValidator {
    token_stream: TokenStream,
}

impl TypeValidator {
    pub fn new() -> Self {
        TypeValidator {
            token_stream: TokenStream::new(),
        }
    }

    pub fn add_path(&mut self, path: &syn::Path, span: Span, arg: &TypeData) {
        let syn_arg = arg.syn_type();
        let type_check = quote_spanned! {span => _ : Box<#path<#syn_arg>>, };
        let tokens = self.token_stream.clone();
        self.token_stream = quote! { #tokens #type_check}
    }

    pub fn add_dyn_path(&mut self, path: &syn::Path, span: Span) {
        let type_check = quote_spanned! {span => _ : Box<dyn #path>, };
        let tokens = self.token_stream.clone();
        self.token_stream = quote! { #tokens #type_check}
    }

    pub fn add_type(&mut self, type_data: &TypeData, span: Span) {
        let path = type_data.syn_type();
        let type_check = quote_spanned! {span => _ : Box<#path>, };
        let tokens = self.token_stream.clone();
        self.token_stream = quote! { #tokens #type_check}
    }

    pub fn add_dyn_type(&mut self, type_data: &TypeData, span: Span) {
        let path = type_data.syn_type();
        let type_check = quote_spanned! {span => _ : Box<dyn #path>, };
        let tokens = self.token_stream.clone();
        self.token_stream = quote! { #tokens #type_check}
    }

    pub fn validate(&self, name: String) -> TokenStream {
        let ident = format_ident!("lockjaw_type_validator_{}", name);
        let types = self.token_stream.clone();
        quote! {
            #[doc(hidden)]
            fn #ident(#types) {}
        }
    }
}
