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
use crate::manifest::{Binding, BuilderModules, MultibindingType};
use crate::nodes::node;
use crate::nodes::node::{ModuleInstance, Node};
use crate::nodes::vec::VecNode;
use crate::type_data::TypeData;
use std::any::Any;

#[derive(Debug, Clone)]
pub struct ProvidesNode {
    pub type_: TypeData,
    pub dependencies: Vec<TypeData>,

    pub module_instance: ModuleInstance,
    pub binding: Binding,
}

impl ProvidesNode {
    pub fn new(
        module_manifest: &BuilderModules,
        module_type: &TypeData,
        binding: &Binding,
    ) -> Vec<Box<dyn Node>> {
        let dependencies = binding
            .dependencies
            .iter()
            .map(|dependency| dependency.type_data.clone())
            .collect();
        let mut type_ = binding.type_data.clone();
        if binding.multibinding_type != MultibindingType::None {
            type_.identifier_suffix = format!("{}", node::get_multibinding_id());
        }
        let mut result: Vec<Box<dyn Node>> = vec![Box::new(ProvidesNode {
            type_: type_.clone(),
            dependencies,
            module_instance: <dyn Node>::get_module_instance(module_manifest, module_type),
            binding: binding.clone(),
        })];
        match binding.multibinding_type {
            MultibindingType::IntoVec => {
                let mut vec_node = VecNode::new(&binding.type_data);
                vec_node.add_binding(&type_, &binding.multibinding_type);
                result.push(vec_node);
            }
            MultibindingType::ElementsIntoVec => {
                let element_type = binding.type_data.args.get(0).unwrap();
                let mut vec_node = VecNode::new(element_type);
                vec_node.add_binding(&type_, &binding.multibinding_type);
                result.push(vec_node);
            }
            _ => {}
        }
        result
    }
}

impl Node for ProvidesNode {
    fn get_name(&self) -> String {
        format!(
            "{}.{} (module provides)",
            self.module_instance.type_.canonical_string_path(),
            self.binding.name
        )
    }

    fn generate_implementation(&self, _graph: &Graph) -> Result<ComponentSections, TokenStream> {
        let mut args = quote! {};
        for arg in &self.binding.dependencies {
            let arg_provider_name = arg.type_data.identifier();
            args = quote! {
                #args  self.#arg_provider_name(),
            }
        }

        let type_path = self.type_.syn_type();

        let name_ident = self.get_identifier();
        let module_method = format_ident!("{}", self.binding.name);
        let invoke_module;

        if self.binding.field_static {
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

    fn get_dependencies(&self) -> Vec<TypeData> {
        self.dependencies.clone()
    }

    fn clone_box(&self) -> Box<dyn Node> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
