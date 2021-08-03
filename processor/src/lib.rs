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
use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::process::Command;

use quote::{quote, quote_spanned};

use crate::manifest::Manifest;
use error::handle_error;

use crate::error::{compile_error, CompileError};
use manifest::ComponentType;
use proc_macro2::Span;
use std::ops::Deref;
use syn::spanned::Spanned;

#[macro_use]
mod log;
mod component_visibles;
mod components;
mod entrypoints;
mod environment;
mod error;
mod graph;
mod injectables;
mod manifest;
mod modules;
mod nodes;
mod parsing;
mod prologue;
mod qualifier;
mod type_data;
mod type_validator;

#[proc_macro_attribute]
pub fn injectable(attr: TokenStream, input: TokenStream) -> TokenStream {
    handle_error(|| injectables::handle_injectable_attribute(attr.into(), input.into()))
}

#[proc_macro_attribute]
pub fn builder_modules(attr: TokenStream, input: TokenStream) -> TokenStream {
    handle_error(|| components::handle_builder_modules_attribute(attr.into(), input.into()))
}

#[proc_macro_attribute]
pub fn component(attr: TokenStream, input: TokenStream) -> TokenStream {
    handle_error(|| {
        components::handle_component_attribute(
            attr.into(),
            input.into(),
            ComponentType::Component,
            false,
        )
    })
}

#[proc_macro_attribute]
pub fn subcomponent(attr: TokenStream, input: TokenStream) -> TokenStream {
    handle_error(|| {
        components::handle_component_attribute(
            attr.into(),
            input.into(),
            ComponentType::Subcomponent,
            false,
        )
    })
}

#[proc_macro_attribute]
pub fn define_component(attr: TokenStream, input: TokenStream) -> TokenStream {
    handle_error(|| {
        components::handle_component_attribute(
            attr.into(),
            input.into(),
            ComponentType::Component,
            true,
        )
    })
}

#[proc_macro_attribute]
pub fn define_subcomponent(attr: TokenStream, input: TokenStream) -> TokenStream {
    handle_error(|| {
        components::handle_component_attribute(
            attr.into(),
            input.into(),
            ComponentType::Subcomponent,
            true,
        )
    })
}

#[proc_macro_attribute]
pub fn entry_point(attr: TokenStream, input: TokenStream) -> TokenStream {
    handle_error(|| entrypoints::handle_entry_point_attribute(attr.into(), input.into()))
}

#[proc_macro_attribute]
pub fn module(attr: TokenStream, input: TokenStream) -> TokenStream {
    handle_error(|| modules::handle_module_attribute(attr.into(), input.into()))
}

#[proc_macro_attribute]
pub fn qualifier(attr: TokenStream, input: TokenStream) -> TokenStream {
    handle_error(|| qualifier::handle_qualifier_attribute(attr.into(), input.into()))
}

#[proc_macro_attribute]
pub fn component_visible(attr: TokenStream, input: TokenStream) -> TokenStream {
    handle_error(|| {
        component_visibles::handle_component_visible_attribute(attr.into(), input.into())
    })
}

#[proc_macro]
pub fn prologue(input: TokenStream) -> TokenStream {
    let input2: proc_macro2::TokenStream = input.into();
    let span = input2.span().resolved_at(Span::call_site());
    let result = quote_spanned! {span=>
        #[cfg(not(any(test,doctest)))]
        lockjaw::private_prologue!(#input2);
        #[cfg(any(test,doctest))]
        lockjaw::private_test_prologue!(#input2);
    };
    result.into()
}

#[proc_macro]
pub fn private_prologue(input: TokenStream) -> TokenStream {
    handle_error(|| {
        // rustdoc --test does not run with #[cfg(test)] and will reach here.
        let for_test = environment::current_crate().eq("lockjaw");
        prologue::handle_prologue(input.into(), for_test)
    })
}

#[proc_macro]
pub fn private_test_prologue(input: TokenStream) -> TokenStream {
    handle_error(|| prologue::handle_prologue(input.into(), true))
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
    root: bool,
}

#[proc_macro]
pub fn private_root_epilogue(input: TokenStream) -> TokenStream {
    handle_error(|| {
        let mut config = EpilogueConfig {
            ..create_epilogue_config(input)
        };
        if environment::current_crate().eq("lockjaw") {
            // rustdoc --test does not run with #[cfg(test)] and will reach here.
            config.for_test = true;
            config.root = true;
        }

        internal_epilogue(config)
    })
}

#[proc_macro]
pub fn private_test_epilogue(input: TokenStream) -> TokenStream {
    handle_error(|| {
        let config = EpilogueConfig {
            for_test: true,
            root: true,
            ..create_epilogue_config(input)
        };
        internal_epilogue(config)
    })
}

fn create_epilogue_config(input: TokenStream) -> EpilogueConfig {
    let set: HashSet<String> = input.into_iter().map(|t| t.to_string()).collect();
    EpilogueConfig {
        debug_output: set.contains("debug_output"),
        for_test: set.contains("test"),
        root: set.contains("root"),
        ..EpilogueConfig::default()
    }
}

fn internal_epilogue(
    config: EpilogueConfig,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    manifest::with_manifest(|mut manifest| {
        let expanded_visibilities = component_visibles::expand_visibilities(&manifest)?;
        manifest.root = config.root;

        let merged_manifest = merge_manifest(&manifest, &config)?;

        //log!("{:#?}", merged_manifest);

        if !config.for_test {
            let out_dir = environment::lockjaw_output_dir()?;
            let manifest_path = format!("{}manifest.pb", out_dir);
            std::fs::create_dir_all(Path::new(&environment::proc_artifact_dir()))
                .expect("cannot create common artifact dir");
            std::fs::create_dir_all(Path::new(&out_dir)).expect("cannot create output dir");

            std::fs::write(
                &manifest_path,
                serde_json::to_string_pretty(manifest.deref()).expect("cannot serialize manifest"),
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
        manifest.clear();

        let (components, messages) =
            components::generate_components(&merged_manifest, config.root)?;

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
            #expanded_visibilities
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

fn merge_manifest(
    manifest: &Manifest,
    config: &EpilogueConfig,
) -> Result<Manifest, proc_macro2::TokenStream> {
    let deps = parsing::get_crate_deps(config.for_test, false);
    //log!("deps: {:?}", deps);

    let mut result = manifest.clone();

    for dep in &deps {
        let manifest_path_file_string =
            format!("{}/{}.manifest_path", environment::proc_artifact_dir(), dep);
        let manifest_path_file = Path::new(&manifest_path_file_string);
        if !manifest_path_file.exists() {
            continue;
        }
        let manifest_path =
            std::fs::read_to_string(manifest_path_file).expect("unable to read manifest path");

        let reader = BufReader::new(File::open(manifest_path).expect("cannot find manifest file"));
        let dep_manifest: Manifest = serde_json::from_reader(reader).expect("cannot read manifest");
        if dep_manifest.root {
            return compile_error(&format!("crate is depending on crate '{}' which already called lockjaw::epilogue!(root).\n\
            epilogue!(root) generates #[define_component] implementations and may only be called once in a binary, typically at the root binary crate", dep));
        }
        result.merge_from(&dep_manifest);
    }
    Ok(result)
}
