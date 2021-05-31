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
use quote::quote;

use crate::graph::ComponentSections;
use crate::graph::Graph;
use crate::manifest::{Binding, ComponentModuleManifest, MultibindingType};
use crate::nodes::component_lifetime::ComponentLifetimeNode;
use crate::nodes::node;
use crate::nodes::node::{ModuleInstance, Node};
use crate::nodes::vec::VecNode;
use crate::type_data::TypeData;
use std::any::Any;

#[derive(Debug, Clone)]
pub struct BindsNode {
    pub type_: TypeData,
    pub dependencies: Vec<TypeData>,

    pub module_instance: ModuleInstance,
    pub binding: Binding,
}

impl BindsNode {
    pub fn new(
        module_manifest: &ComponentModuleManifest,
        module_type: &TypeData,
        binding: &Binding,
    ) -> Vec<Box<dyn Node>> {
        let dependencies = binding
            .dependencies
            .iter()
            .map(|dependency| dependency.type_data.clone())
            .collect();
        let mut type_ = ComponentLifetimeNode::component_lifetime_type(&binding.type_data);
        if binding.multibinding_type != MultibindingType::None {
            type_.identifier_suffix = format!("{}", node::get_multibinding_id());
        }
        let mut result: Vec<Box<dyn Node>> = vec![Box::new(BindsNode {
            type_: type_.clone(),
            dependencies,
            module_instance: <dyn Node>::get_module_instance(module_manifest, module_type),
            binding: binding.clone(),
        })];
        match binding.multibinding_type {
            MultibindingType::IntoVec => {
                let mut vec_node = VecNode::new(&type_);
                vec_node.add_binding(&type_, &binding.multibinding_type);
                result.push(vec_node);
            }
            MultibindingType::ElementsIntoVec => {
                panic!("unexpected #[elements_into_vec] for #[binds]")
            }
            _ => {}
        }
        result
    }
}

impl Node for BindsNode {
    fn get_name(&self) -> String {
        format!(
            "{}.{} (module binds)",
            self.module_instance.type_.canonical_string_path(),
            self.binding.name
        )
    }

    fn generate_implementation(&self, _graph: &Graph) -> Result<ComponentSections, TokenStream> {
        let arg = self
            .binding
            .dependencies
            .first()
            .expect("binds must have one arg");
        let arg_provider_name = arg.type_data.identifier();

        let name_ident = self.get_identifier();
        let type_path = self.type_.syn_type();

        let mut result = ComponentSections::new();
        if arg.type_data.field_ref {
            result.add_methods(quote! {
                fn #name_ident(&'_ self) -> #type_path{
                    lockjaw::ComponentLifetime::Ref(self.#arg_provider_name())
                }
            });
        } else {
            result.add_methods(quote! {
                fn #name_ident(&'_ self) -> #type_path{
                    lockjaw::ComponentLifetime::Val(Box::new(self.#arg_provider_name()))
                }
            });
        }
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
