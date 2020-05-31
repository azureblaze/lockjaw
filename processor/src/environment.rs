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
use crate::error::CompileError;
#[allow(unused)]
use crate::log;
use proc_macro2::TokenStream;
use std::path::Path;
use syn::spanned::Spanned;

pub fn current_crate() -> String {
    std::env::var("CARGO_CRATE_NAME").expect("missing crate name env var")
}

/// Returns the current file being compiled.
#[cfg(nightly)]
pub fn current_file_path(input: TokenStream) -> Result<String, TokenStream> {
    let span = input.span();
    let item: syn::LitStr =
        syn::parse2(input).map_spanned_compile_error(span, "input must be string literal")?;
    let path = item
        .span()
        .unwrap()
        .source_file()
        .path()
        .as_path()
        .to_str()
        .expect("cannot extract path")
        .replace("\\", "/");
    if path.starts_with("src/") {
        return Ok(path.split_at(4).1.to_string());
    }
    return Ok(path
        .split_at(path.find("/src/").expect("file path not under src") + 5)
        .1
        .to_string());
}

/// Returns the current file being compiled.
#[cfg(not(nightly))]
pub fn current_file_path(input: TokenStream) -> Result<String, TokenStream> {
    let span = input.span();
    let item: syn::LitStr = syn::parse2(input)
        .map_spanned_compile_error(span, "passed file path not string literal")?;
    Ok(item.value().replace("\\", "/"))
}

pub fn get_base_path(path: &str) -> Result<String, TokenStream> {
    let src_path = Path::new(path);
    let stem = src_path
        .file_stem()
        .map_compile_error("no source file stem")?
        .to_str()
        .expect("cannot convert file stem to utf-8");
    if src_path
        .parent()
        .map_compile_error("missing source parent")?
        .eq(Path::new(""))
    {
        match stem {
            "main" => Ok(String::new()),
            "lib" => Ok(String::new()),
            _ => Ok(stem.to_string()),
        }
    } else {
        let parent_path = src_path
            .parent()
            .map_compile_error("missing source parent")?
            .components()
            .map(|c| {
                c.as_os_str()
                    .to_str()
                    .expect("unable to convert os path to utf-8")
                    .to_string()
            })
            .collect::<Vec<String>>()
            .join("::");
        match stem {
            "mod" => Ok(parent_path),
            _ => Ok(parent_path + "::" + &stem.to_string()),
        }
    }
}

#[test]
fn get_base_path_main_returns_empty() {
    assert_eq!(get_base_path("main.rs").unwrap(), "");
}

#[test]
fn get_base_path_lib_returns_empty() {
    assert_eq!(get_base_path("lib.rs").unwrap(), "");
}

#[test]
fn get_base_path_root_file_returns_file() {
    assert_eq!(get_base_path("file.rs").unwrap(), "file");
}

#[test]
fn get_base_path_nested_mod_returns_dir() {
    assert_eq!(get_base_path("foo/mod.rs").unwrap(), "foo");
}

#[test]
fn get_base_path_nested_file_returns_dir_and_file() {
    assert_eq!(get_base_path("foo/bar.rs").unwrap(), "foo::bar");
}

/// Returns the common artifacts directory where all crates can read/write.
///
/// Each crate will write its [lockjaw_output_dir] here so dependent crate can find them.
pub fn proc_artifact_dir() -> String {
    return format!("{}/lockjaw_artifacts/", env!("PROC_ARTIFACT_DIR"));
}

/// Returns the output directory for the current crate.
pub fn lockjaw_output_dir() -> Result<String, TokenStream> {
    let out_dir = std::env::var("OUT_DIR")
        .map_compile_error("output dir not found. Call lockjaw::build_script in build.rs")?;
    return Ok(format!("{}/lockjaw/", out_dir));
}
