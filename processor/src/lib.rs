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

use quote::quote;

use error::handle_error;

use crate::error::CompileError;
use lockjaw_common::environment::{current_crate, current_package};
use lockjaw_common::manifest::LockjawPackage;
use lockjaw_common::manifest::{ComponentType, DepManifests, Manifest};
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
mod qualifier;
mod type_data;
mod type_validator;

#[proc_macro_attribute]
pub fn injectable(attr: TokenStream, input: TokenStream) -> TokenStream {
    handle_error(|| injectables::handle_injectable_attribute(attr.into(), input.into()))
}

#[proc_macro_attribute]
pub fn injectable_inject(_attr: TokenStream, _input: TokenStream) -> TokenStream {
    doc_proc_macro("#[inject] should only annotate an item under a #[injectable] item. This attribute macro is for documentation purpose only and should not be called directly.")
}

#[proc_macro_attribute]
pub fn injectable_factory(_attr: TokenStream, _input: TokenStream) -> TokenStream {
    doc_proc_macro("#[factory] should only annotate an item under a #[injectable] item. This attribute macro is for documentation purpose only and should not be called directly.")
}

#[proc_macro_attribute]
pub fn builder_modules(attr: TokenStream, input: TokenStream) -> TokenStream {
    handle_error(|| components::handle_builder_modules_attribute(attr.into(), input.into()))
}

#[proc_macro_attribute]
pub fn component(attr: TokenStream, input: TokenStream) -> TokenStream {
    handle_error(|| {
        components::handle_component_attribute(attr.into(), input.into(), ComponentType::Component)
    })
}

#[proc_macro_attribute]
pub fn subcomponent(attr: TokenStream, input: TokenStream) -> TokenStream {
    handle_error(|| {
        components::handle_component_attribute(
            attr.into(),
            input.into(),
            ComponentType::Subcomponent,
        )
    })
}

#[proc_macro_attribute]
pub fn define_component(attr: TokenStream, input: TokenStream) -> TokenStream {
    handle_error(|| {
        components::handle_component_attribute(attr.into(), input.into(), ComponentType::Component)
    })
}

#[proc_macro_attribute]
pub fn define_subcomponent(attr: TokenStream, input: TokenStream) -> TokenStream {
    handle_error(|| {
        components::handle_component_attribute(
            attr.into(),
            input.into(),
            ComponentType::Subcomponent,
        )
    })
}

#[proc_macro_attribute]
pub fn component_qualified(_attr: TokenStream, _input: TokenStream) -> TokenStream {
    doc_proc_macro("#[qualified] should only annotate an item under a #[component]/#[subcomponent]/#[define_component]/#[define_subcomponent] item. This attribute macro is for documentation purpose only and should not be called directly.")
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
pub fn module_provides(_attr: TokenStream, _input: TokenStream) -> TokenStream {
    doc_proc_macro("#[provides] should only annotate an item under a #[module] item. This attribute macro is for documentation purpose only and should not be called directly.")
}

#[proc_macro_attribute]
pub fn module_binds(_attr: TokenStream, _input: TokenStream) -> TokenStream {
    doc_proc_macro("#[binds] should only annotate an item under a #[module] item. This attribute macro is for documentation purpose only and should not be called directly.")
}

#[proc_macro_attribute]
pub fn module_binds_option_of(_attr: TokenStream, _input: TokenStream) -> TokenStream {
    doc_proc_macro("#[binds_option_of] should only annotate an item under a #[module] item. This attribute macro is for documentation purpose only and should not be called directly.")
}

#[proc_macro_attribute]
pub fn module_multibinds(_attr: TokenStream, _input: TokenStream) -> TokenStream {
    doc_proc_macro("#[multibinds] should only annotate an item under a #[module] item. This attribute macro is for documentation purpose only and should not be called directly.")
}

#[proc_macro_attribute]
pub fn module_into_vec(_attr: TokenStream, _input: TokenStream) -> TokenStream {
    doc_proc_macro("#[into_vec] should only annotate an item under a #[module] item. This attribute macro is for documentation purpose only and should not be called directly.")
}

#[proc_macro_attribute]
pub fn module_elements_into_vec(_attr: TokenStream, _input: TokenStream) -> TokenStream {
    doc_proc_macro("#[elements_into_vec] should only annotate an item under a #[module] item. This attribute macro is for documentation purpose only and should not be called directly.")
}

#[proc_macro_attribute]
pub fn module_into_map(_attr: TokenStream, _input: TokenStream) -> TokenStream {
    doc_proc_macro("#[into_map] should only annotate an item under a #[module] item. This attribute macro is for documentation purpose only and should not be called directly.")
}

#[proc_macro_attribute]
pub fn module_qualified(_attr: TokenStream, _input: TokenStream) -> TokenStream {
    doc_proc_macro("#[qualified] should only annotate an item under a #[module] item. This attribute macro is for documentation purpose only and should not be called directly.")
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
        if current_package().eq("lockjaw") {
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
        for_test: false,
        root: std::env::var("CARGO_BIN_NAME").is_ok(),
        ..EpilogueConfig::default()
    }
}

fn internal_epilogue(
    config: EpilogueConfig,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    let merged_manifest = merge_manifest(&config)?;
    let expanded_visibilities = component_visibles::expand_visibilities(&merged_manifest)?;

    let (components, initiazers, messages) =
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

    let root_component_initializer = if config.root {
        quote! {
            #[doc(hidden)]
            #[no_mangle]
            #[allow(non_snake_case)]
            pub(crate) fn lockjaw_init_root_components(){
                #initiazers
            }
        }
    } else {
        quote! {}
    };

    let result = quote! {
        #expanded_visibilities
        #components
        #path_test

        #root_component_initializer
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
            current_crate()
        );
        log!(
            "writing debug output to file:///{}",
            path.replace("\\", "/")
        );
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
}

fn merge_manifest(config: &EpilogueConfig) -> Result<Manifest, proc_macro2::TokenStream> {
    let mut result: Manifest = Manifest::new();
    if let Ok(manifest) = std::env::var("LOCKJAW_TRYBUILD_PATH") {
        let test_manifest = lockjaw_common::manifest_parser::parse_manifest(&LockjawPackage {
            id: "".to_string(),
            name: std::env::var("CARGO_PKG_NAME").unwrap().replace("-", "_"),
            src_path: manifest,
            direct_prod_crate_deps: vec![],
            direct_test_crate_deps: vec![],
        })
        .test_manifest;
        result.merge_from(&test_manifest);
        return Ok(result);
    }

    if let Ok(manifest) = std::env::var("LOCKJAW_DEP_MANIFEST") {
        let reader = BufReader::new(File::open(manifest).expect("cannot find manifest file"));
        let dep_manifest: DepManifests =
            serde_json::from_reader(reader).expect("cannot read manifest");
        if config.for_test {
            for dep in &dep_manifest.test_manifest {
                result.merge_from(dep)
            }
        } else {
            for dep in &dep_manifest.prod_manifest {
                result.merge_from(dep)
            }
        }
        if let Ok(bin_name) = std::env::var("CARGO_BIN_NAME") {
            let root_manifest = dep_manifest
                .root_manifests
                .get(&bin_name)
                .expect("CARGO_BIN_NAME not in manifest");
            if config.for_test {
                result.merge_from(&root_manifest.test_manifest);
            } else {
                result.merge_from(&root_manifest.prod_manifest);
            }
        } else {
            if config.for_test {
                let test_target = std::env::var("CARGO_CRATE_NAME").unwrap();
                let test_manifest = &dep_manifest
                    .root_manifests
                    .get(&test_target)
                    .unwrap()
                    .test_manifest;

                //log!("test manifest: {:#?}", test_manifest);
                result.merge_from(&test_manifest);
            } else {
                result.merge_from(
                    &dep_manifest
                        .root_manifests
                        .get(&std::env::var("CARGO_CRATE_NAME").unwrap())
                        .expect("CARGO_CRATE_NAME not in manifest")
                        .prod_manifest,
                )
            }
        }
    } else {
        return Err(
            quote! { compile_error!("manifest missing, is the lockjaw::build_script called in build.rs?");},
        );
    }
    Ok(result)
}

fn doc_proc_macro(message: &str) -> TokenStream {
    (quote! { compile_error!(#message)}).into()
}
