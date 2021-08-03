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

use crate::component_visibles;
use crate::graph::ComponentSections;
use crate::graph::Graph;
use crate::manifest::{Binding, BuilderModules, MultibindingType};
use crate::nodes::component_lifetime::ComponentLifetimeNode;
use crate::nodes::map::MapNode;
use crate::nodes::node;
use crate::nodes::node::{DependencyData, ModuleInstance, Node};
use crate::nodes::vec::VecNode;
use crate::type_data::TypeData;
use std::any::Any;

#[derive(Debug, Clone)]
pub struct BindsNode {
    pub type_: TypeData,
    pub dependency: TypeData,

    pub module_instance: ModuleInstance,
    pub binding: Binding,
}

impl BindsNode {
    pub fn new(
        module_manifest: &BuilderModules,
        module_type: &TypeData,
        binding: &Binding,
    ) -> Result<Vec<Box<dyn Node>>, TokenStream> {
        let mut type_ = ComponentLifetimeNode::component_lifetime_type(&binding.type_data);
        if binding.multibinding_type != MultibindingType::None {
            type_.identifier_suffix = format!("{}", node::get_multibinding_id());
        }

        let mut result: Vec<Box<dyn Node>> = vec![Box::new(BindsNode {
            type_: type_.clone(),
            dependency: binding
                .dependencies
                .first()
                .expect("binds must have one arg")
                .type_data
                .clone(),
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
            MultibindingType::IntoMap => {
                let mut map_node = MapNode::new(&binding.map_key, &binding.type_data)?;
                map_node.add_binding(&binding.map_key, &type_);
                result.push(map_node);
            }
            _ => {}
        }
        Ok(result)
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

    fn generate_implementation(&self, graph: &Graph) -> Result<ComponentSections, TokenStream> {
        let arg_provider_name = self.dependency.identifier();

        let name_ident = self.get_identifier();
        let type_path = component_visibles::visible_type(graph.manifest, &self.type_).syn_type();

        let mut result = ComponentSections::new();
        if self.dependency.field_ref {
            result.add_methods(quote! {
                fn #name_ident(&'_ self) -> #type_path{
                    lockjaw::Cl::Ref(self.#arg_provider_name())
                }
            });
        } else {
            result.add_methods(quote! {
                fn #name_ident(&'_ self) -> #type_path{
                    lockjaw::Cl::Val(Box::new(self.#arg_provider_name()))
                }
            });
        }
        Ok(result)
    }

    fn get_type(&self) -> &TypeData {
        &self.type_
    }

    fn get_dependencies(&self) -> Vec<DependencyData> {
        vec![DependencyData::from_type(&self.dependency)]
    }

    fn clone_box(&self) -> Box<dyn Node> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}
