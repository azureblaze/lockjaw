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

use crate::environment::cargo_manifest_dir;
use crate::error::{spanned_compile_error, CompileError};
use proc_macro2::TokenStream;
use quote::quote;
use std::cell::RefCell;
use syn::parse::Parser;
use syn::spanned::Spanned;

thread_local! {
    static TEST_SOURCE :RefCell<String> = RefCell::new(String::new());
}

pub fn get_test_source() -> String {
    TEST_SOURCE.with(|test_source| test_source.borrow().clone())
}

pub fn handle_prologue(input: TokenStream, mut for_test: bool) -> Result<TokenStream, TokenStream> {
    let args = literal_to_strings(&input)?;
    if args.len() == 0 {
        return spanned_compile_error(
            input.span(),
            "current source file name from cargo.toml expected",
        );
    }
    if args.len() > 3 {
        return spanned_compile_error(
            input.span(),
            "too many arguments. expected current source file name, and optional path to source root (src/, tests/, etc.)",
        );
    }

    let filename = args.get(0).unwrap().replace("\\", "/");
    let full_path = format!("{}/{}", cargo_manifest_dir(), filename);
    if args.len() >= 3 && args.get(2).unwrap() == "test" {
        for_test = true;
    }
    if for_test {
        TEST_SOURCE.with(|test_source| *test_source.borrow_mut() = full_path.clone())
    }

    Ok(quote! {})
}

fn literal_to_strings(input: &TokenStream) -> Result<Vec<String>, TokenStream> {
    let parser = syn::punctuated::Punctuated::<syn::LitStr, syn::Token![,]>::parse_terminated;
    let punctuated = parser
        .parse2(input.clone())
        .map_spanned_compile_error(input.span(), "comma seperated string literals expected")?;
    Ok(punctuated.iter().map(syn::LitStr::value).collect())
}
