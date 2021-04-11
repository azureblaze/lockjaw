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

#![cfg_attr(nightly, feature(proc_macro_span, proc_macro_diagnostic))]

use proc_macro;
use proc_macro::TokenStream;
use std::cell::RefCell;
use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::process::Command;

use quote::quote;
use regex::Regex;
use syn::spanned::Spanned;

use crate::manifest::Manifest;
use error::handle_error;

use crate::error::CompileError;

#[macro_use]
mod log;
mod components;
mod environment;
mod error;
mod graph;
mod injectables;
mod manifest;
mod manifests;
mod modules;
mod nodes;
mod parsing;
thread_local! {
    static MANIFEST :RefCell<Manifest> = RefCell::new(Manifest::new());
}

#[proc_macro_attribute]
pub fn injectable(attr: TokenStream, input: TokenStream) -> TokenStream {
    handle_error(|| injectables::handle_injectable_attribute(attr.into(), input.into()))
}

#[proc_macro_attribute]
pub fn component_module_manifest(attr: TokenStream, input: TokenStream) -> TokenStream {
    handle_error(|| {
        components::handle_component_module_manifest_attribute(attr.into(), input.into())
    })
}

#[proc_macro_attribute]
pub fn component(attr: TokenStream, input: TokenStream) -> TokenStream {
    handle_error(|| components::handle_component_attribute(attr.into(), input.into()))
}

#[proc_macro_attribute]
pub fn module(attr: TokenStream, input: TokenStream) -> TokenStream {
    handle_error(|| modules::handle_module_attribute(attr.into(), input.into()))
}

#[proc_macro_attribute]
pub fn provides(_attr: TokenStream, input: TokenStream) -> TokenStream {
    input
}

#[proc_macro]
pub fn mod_epilogue(input: TokenStream) -> TokenStream {
    handle_error(|| {
        let path = environment::current_file_path(input.into())?;
        extend_local_manifest(&path)?;
        let path_test = quote! {
            #[test]
            fn mod_epilogue_path_matches_macro_input(){
                let crate_name = module_path!().split("::").next().unwrap();
                let mod_path = format!("{}::{}",crate_name, #path);
                assert_eq!(module_path!(), mod_path,
                    "path supplied to epilogue!() does not match actual path");
            }
        };
        Ok(path_test)
    })
}

#[proc_macro]
pub fn test_mod_epilogue(input: TokenStream) -> TokenStream {
    handle_error(|| {
        let input2: proc_macro2::TokenStream = input.into();
        let span = input2.span();
        let path: syn::LitStr =
            syn::parse2(input2).map_spanned_compile_error(span, "input must be string literal")?;
        extend_local_manifest(&path.value().replace("\\", "/"))?;
        Ok(quote! {})
    })
}

fn extend_local_manifest(path: &str) -> Result<(), TokenStream> {
    MANIFEST.with(|m| {
        let mut manifest = m.borrow_mut();
        manifest
            .injectables
            .extend(injectables::generate_manifest(path));
        manifest
            .components
            .extend(components::generate_component_manifest(path));
        manifest
            .component_module_manifests
            .extend(components::generate_component_module_manifest(path));
        manifest.modules.extend(modules::generate_manifest(path));
        Ok(())
    })
}

#[proc_macro]
pub fn epilogue(input: TokenStream) -> TokenStream {
    let input2: proc_macro2::TokenStream = input.into();
    let result = quote! {
        #[cfg(not(any(test,doctest)))]
        lockjaw::private_root_epilogue!(#input2);
        #[cfg(any(test,doctest))]
        lockjaw::private_test_epilogue!(#input2);
    };
    result.into()
}

#[derive(Default)]
struct EpilogueConfig {
    for_test: bool,
    debug_output: bool,
}

#[proc_macro]
pub fn private_root_epilogue(input: TokenStream) -> TokenStream {
    handle_error(|| {
        let config = EpilogueConfig {
            // rustdoc --test does not run with #[cfg(test)] and will reach here.
            for_test: environment::current_crate().eq("lockjaw"),
            ..create_epilogue_config(input)
        };
        internal_epilogue(config)
    })
}

#[proc_macro]
pub fn private_test_epilogue(input: TokenStream) -> TokenStream {
    handle_error(|| {
        let config = EpilogueConfig {
            for_test: true,
            ..create_epilogue_config(input)
        };
        internal_epilogue(config)
    })
}

fn create_epilogue_config(input: TokenStream) -> EpilogueConfig {
    let set: HashSet<String> = input.into_iter().map(|t| t.to_string()).collect();
    EpilogueConfig {
        debug_output: set.contains("debug_output"),
        ..EpilogueConfig::default()
    }
}

fn internal_epilogue(
    config: EpilogueConfig,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    extend_local_manifest("")?;

    MANIFEST.with(|manifest| {
        let merged_manifest = merge_manifest(&manifest.borrow_mut(), config.for_test);
        manifest.borrow_mut().clear();
        if !config.for_test {
            let out_dir = environment::lockjaw_output_dir()?;
            let manifest_path = format!("{}manifest.pb", out_dir);
            std::fs::create_dir_all(Path::new(&environment::proc_artifact_dir()))
                .expect("cannot create common artifact dir");
            std::fs::create_dir_all(Path::new(&out_dir)).expect("cannot create output dir");

            std::fs::write(
                &manifest_path,
                serde_json::to_string_pretty(&merged_manifest).expect("cannot serialize manifest"),
            )
            .expect("cannot write manifest");

            std::fs::write(
                Path::new(&format!(
                    "{}/{}.manifest_path",
                    environment::proc_artifact_dir(),
                    environment::current_crate()
                )),
                &manifest_path,
            )
            .expect("cannot write manifest path");
        }
        let (components, messages) = components::generate_components(&merged_manifest)?;

        let path_test;
        if config.for_test {
            path_test = quote! {}
        } else {
            path_test = quote! {
                #[test]
                fn epilogue_invoked_at_crate_root(){
                    let crate_name = module_path!().split("::").next().unwrap();
                    let mod_path = crate_name.to_owned();
                    assert_eq!(
                        module_path!(),
                        mod_path,
                        "lockjaw::epilogue!() called in a file other than lib.rs or main.rs"
                    );
                }
            };
        }

        let result = quote! {
            #components
            #path_test
        };

        if config.debug_output {
            let mut content = format!("/* manifest:\n{:#?}\n*/\n", merged_manifest);
            for message in messages {
                content.push_str(&format!("/*\n{}\n*/\n", message));
            }
            content.push_str(&result.to_string());
            let path = format!(
                "{}debug_{}.rs",
                environment::lockjaw_output_dir()?,
                environment::current_crate()
            );
            log!("writing debug output to {}", path);
            std::fs::create_dir_all(Path::new(&environment::lockjaw_output_dir()?))
                .expect("cannot create output dir");
            std::fs::write(Path::new(&path), &content)
                .expect(&format!("cannot write debug output to {}", path));

            Command::new("rustfmt")
                .arg(&path)
                .output()
                .map_compile_error("unable to format output")?;

            Ok(quote! {
                std::include!(#path);
            })
        } else {
            Ok(result)
        }
    })
}

fn deps(for_test: bool) -> HashSet<String> {
    let tree = String::from_utf8(
        Command::new("cargo")
            .current_dir(std::env::var("CARGO_MANIFEST_DIR").expect("missing manifest dir"))
            .arg("tree")
            .arg("--prefix")
            .arg("depth")
            .arg("-e")
            .arg(if for_test { "normal,dev" } else { "normal" })
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .split("\n")
    .map(|s| s.to_string())
    .collect::<Vec<String>>();
    let mut deps = HashSet::<String>::new();
    let pattern = Regex::new(r"(?P<depth>\d+)(?P<crate>[A-Za-z_][A-Za-z0-9_\-]*).*").unwrap();
    for item in tree {
        if item.is_empty() {
            continue;
        }
        let captures = pattern.captures(&item).unwrap();
        if captures["depth"].ne("1") {
            continue;
        }
        deps.insert(captures["crate"].replace("-", "_").to_string());
    }
    deps
}

fn merge_manifest(manifest: &Manifest, for_test: bool) -> Manifest {
    let deps = deps(for_test);
    //log!("deps: {:?}", deps);

    let mut result = manifest.clone();

    for dep in &deps {
        if result.merged_crates.contains(dep) {
            continue;
        }
        result.merged_crates.push(dep.clone());
        let manifest_path_file_string =
            format!("{}/{}.manifest_path", environment::proc_artifact_dir(), dep);
        let manifest_path_file = Path::new(&manifest_path_file_string);
        if !manifest_path_file.exists() {
            continue;
        }
        let manifest_path =
            std::fs::read_to_string(manifest_path_file).expect("unable to read manifest path");

        let reader = BufReader::new(File::open(manifest_path).expect("cannot find manifest file"));
        result.merge_from(&mut serde_json::from_reader(reader).expect("cannot read manifest"));
    }
    result
}
