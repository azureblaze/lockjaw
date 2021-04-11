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
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::graph::ComponentSections;
use crate::graph::Graph;
use crate::manifest::{ComponentModuleManifest, Provider};
use crate::nodes::binds::BindsNode;
use crate::nodes::node::{ModuleInstance, Node};
use crate::type_data::TypeData;

#[derive(Debug, Clone)]
pub struct ProvidesNode {
    pub type_: TypeData,
    pub dependencies: Vec<TypeData>,
    pub scoped: bool,

    pub module_instance: ModuleInstance,
    pub provider: Provider,
}

impl ProvidesNode {
    pub fn new(
        module_manifest: &ComponentModuleManifest,
        module_type: &TypeData,
        provider: &Provider,
    ) -> Vec<Box<dyn Node>> {
        let dependencies = provider
            .dependencies
            .iter()
            .map(|dependency| dependency.type_data.clone())
            .collect();
        let node: Box<dyn Node>;
        if provider.binds {
            node = Box::new(BindsNode {
                type_: <dyn Node>::maybe_scoped_type(&provider.type_data),
                dependencies,
                scoped: false,
                module_instance: <dyn Node>::get_module_instance(module_manifest, module_type),
                provider: provider.clone(),
            });
        } else {
            node = Box::new(ProvidesNode {
                type_: provider.type_data.clone(),
                dependencies,
                scoped: false,
                module_instance: <dyn Node>::get_module_instance(module_manifest, module_type),
                provider: provider.clone(),
            });
        }
        <dyn Node>::generate_node_variants(node)
    }
}

impl Node for ProvidesNode {
    fn get_name(&self) -> String {
        format!(
            "{}.{} (module provides)",
            self.module_instance.type_.canonical_string_path(),
            self.provider.name
        )
    }

    fn generate_provider(&self, _graph: &Graph) -> Result<ComponentSections, TokenStream> {
        let mut args = quote! {};
        for arg in &self.provider.dependencies {
            let arg_provider_name = arg.type_data.identifier();
            args = quote! {
                #args  self.#arg_provider_name(),
            }
        }

        let type_path = self.type_.syn_type();

        let name_ident = self.get_identifier();
        let module_method = format_ident!("{}", self.provider.name);
        let invoke_module;

        if self.provider.field_static {
            let module_path = self.module_instance.type_.syn_type();
            invoke_module = quote! {#module_path::#module_method(#args)}
        } else {
            let module_name = self.module_instance.name.clone();
            invoke_module = quote! {self.#module_name.#module_method(#args)}
        }
        let mut result = ComponentSections::new();
        result.add_methods(quote! {
            fn #name_ident(&'_ self) -> #type_path{
                #invoke_module
            }
        });
        Ok(result)
    }

    fn get_type(&self) -> &TypeData {
        &self.type_
    }

    fn get_dependencies(&self) -> &Vec<TypeData> {
        &self.dependencies
    }

    fn is_scoped(&self) -> bool {
        self.scoped
    }

    fn set_scoped(&mut self, scoped: bool) {
        self.scoped = scoped;
    }

    fn clone_box(&self) -> Box<dyn Node> {
        Box::new(self.clone())
    }
}
