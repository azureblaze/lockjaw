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
use std::fs::File;
use std::path::Path;
use syn::parse::Parser;
use syn::spanned::Spanned;

static mut BASE_PATH: String = String::new();

pub fn prologue_check() -> TokenStream {
    quote! {
        impl lockjaw_error_call_prologue_macro_before_any_other_use {}
    }
}

pub fn get_base_path() -> String {
    unsafe { BASE_PATH.clone() }
}

pub fn handle_prologue(input: TokenStream) -> Result<TokenStream, TokenStream> {
    let args = literal_to_strings(&input)?;
    if args.len() == 0 {
        return spanned_compile_error(
            input.span(),
            "current source file name from cargo.toml expected",
        );
    }
    if args.len() > 2 {
        return spanned_compile_error(
            input.span(),
            "too many arguments. expected current source file name, and optional path to source root (src/, tests/, etc.)",
        );
    }

    let filename = args.get(0).unwrap().replace("\\", "/");
    let full_path = format!("{}/{}", cargo_manifest_dir(), filename);
    let _source_file = File::open(&full_path).map_spanned_compile_error(
        input.span(),
        &format!("unable to find the source file at {}", &full_path),
    )?;

    let mod_path = if args.len() == 2 {
        args.get(1).unwrap().replace("\\", "/")
    } else {
        get_mod_path(&filename)?
    };

    unsafe { BASE_PATH = mod_path.clone() };

    Ok(quote! {
        struct lockjaw_error_call_prologue_macro_before_any_other_use;
        #[test]
        fn prologue_path_matches_macro_input(){
            let mut iter = module_path!().split("::");
            iter.next();
            let path = iter.collect::<Vec<&str>>().join("::");
            assert_eq!(path, #mod_path,
            "source path supplied to prologue!() does not match actual path");
        }
    })
}

fn get_mod_path(file_path: &str) -> Result<String, TokenStream> {
    let source_root = get_source_root(&file_path).map_compile_error(
        "unable to infer source root. \
            pass in the source root as second arg if the project layout does not follow \
            https://doc.rust-lang.org/cargo/guide/project-layout.html",
    )?;

    if source_root != "src/" {
        return Ok("".to_owned());
    }

    let result = Path::new(file_path)
        .strip_prefix(Path::new(&source_root))
        .unwrap()
        .parent()
        .unwrap()
        .to_str()
        .unwrap()
        .replace("/", "::");
    let stem = Path::new(file_path).file_stem().unwrap().to_str().unwrap();
    match stem {
        "lib" | "main" | "mod" => Ok(result),
        _ => {
            if result.is_empty() {
                Ok(stem.to_owned())
            } else {
                Ok(format!("{}::{}", result, stem))
            }
        }
    }
}

#[test]
pub fn get_mod_path_main_empty() {
    assert_eq!(get_mod_path("src/main.rs").unwrap(), "");
}

#[test]
pub fn get_mod_path_lib_empty() {
    assert_eq!(get_mod_path("src/lib.rs").unwrap(), "");
}

#[test]
pub fn get_mod_path_file_file() {
    assert_eq!(get_mod_path("src/foo.rs").unwrap(), "foo");
}

#[test]
pub fn get_mod_path_mod_mod() {
    assert_eq!(get_mod_path("src/foo/mod.rs").unwrap(), "foo");
}

#[test]
pub fn get_mod_path_mod_file_file() {
    assert_eq!(get_mod_path("src/foo/bar.rs").unwrap(), "foo::bar");
}

fn get_source_root(file_path: &str) -> Option<String> {
    if file_path.starts_with("src/bin/") {
        return Some("src/bin/".to_owned());
    }
    if file_path.starts_with("src/") {
        return Some("src/".to_owned());
    }
    if file_path.starts_with("benches/") {
        return Some("benches/".to_owned());
    }
    if file_path.starts_with("examples/") {
        return Some("examples/".to_owned());
    }
    if file_path.starts_with("tests/") {
        return Some("tests/".to_owned());
    }
    None
}

fn literal_to_strings(input: &TokenStream) -> Result<Vec<String>, TokenStream> {
    let parser = syn::punctuated::Punctuated::<syn::LitStr, syn::Token![,]>::parse_terminated;
    let punctuated = parser
        .parse2(input.clone())
        .map_spanned_compile_error(input.span(), "comma seperated string literals expected")?;
    Ok(punctuated.iter().map(syn::LitStr::value).collect())
}
