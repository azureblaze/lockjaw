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
use crate::component_visibles;
use crate::graph::ComponentSections;
use crate::graph::Graph;
use crate::manifest::TypeRoot;
use crate::nodes::node::{DependencyData, Node};
use crate::type_data::TypeData;
use proc_macro2::TokenStream;
use quote::quote;
use std::any::Any;
use std::collections::HashMap;
use syn::Ident;

#[derive(Debug)]
pub struct ComponentLifetimeNode {
    pub type_: TypeData,
    pub dependencies: Vec<TypeData>,

    pub inner: TypeData,
}

impl ComponentLifetimeNode {
    pub fn for_type(
        map: &HashMap<Ident, Box<dyn Node>>,
        type_: &TypeData,
    ) -> Option<Box<dyn Node>> {
        let mut inner = type_.args[0].clone();
        if !inner.field_ref {
            let mut ref_type = inner.clone();
            ref_type.field_ref = true;
            if map.contains_key(&ref_type.identifier()) {
                inner = ref_type;
            }
        }
        Some(Box::new(ComponentLifetimeNode {
            type_: ComponentLifetimeNode::component_lifetime_type(&type_.args[0]),
            dependencies: vec![inner.clone()],

            inner: inner.clone(),
        }))
    }

    pub fn component_lifetime_type(type_: &TypeData) -> TypeData {
        let mut cl_type = TypeData::new();
        cl_type.root = TypeRoot::GLOBAL;
        cl_type.path = "lockjaw::Cl".to_string();
        cl_type.args.push(type_.clone());
        cl_type
    }
}

impl Clone for ComponentLifetimeNode {
    fn clone(&self) -> Self {
        ComponentLifetimeNode {
            type_: self.type_.clone(),
            dependencies: self.dependencies.clone(),
            inner: self.inner.clone(),
        }
    }
}

impl Node for ComponentLifetimeNode {
    fn get_name(&self) -> String {
        format!("Cl{}", self.type_.canonical_string_path())
    }

    fn generate_implementation(&self, graph: &Graph) -> Result<ComponentSections, TokenStream> {
        let arg_provider_name = self.inner.identifier();
        let name_ident = self.get_identifier();
        let type_path = component_visibles::visible_type(graph.manifest, &self.type_).syn_type();

        let mut result = ComponentSections::new();
        if self.inner.field_ref {
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

    fn merge(&self, new_node: &dyn Node) -> Result<Box<dyn Node>, TokenStream> {
        if self
            .type_
            .canonical_string_path()
            .eq(&new_node.get_type().canonical_string_path())
        {
            return Ok(self.clone_box());
        }
        <dyn Node>::duplicated(self, new_node)
    }

    fn can_depend(
        &self,
        _target_node: &dyn Node,
        _ancestors: &Vec<String>,
    ) -> Result<(), TokenStream> {
        Ok(())
    }

    fn get_type(&self) -> &TypeData {
        &self.type_
    }

    fn get_dependencies(&self) -> Vec<DependencyData> {
        DependencyData::from_type_vec(&self.dependencies)
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
