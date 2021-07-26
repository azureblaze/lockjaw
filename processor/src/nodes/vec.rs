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

use crate::graph::{ComponentSections, Graph};
use crate::manifest::{MultibindingType, TypeRoot};
use crate::nodes::node::{DependencyData, Node};
use crate::type_data::TypeData;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::any::{Any, TypeId};
use std::iter::Extend;

#[derive(Debug, Clone)]
pub struct VecBinding {
    pub type_data: TypeData,
    pub multibinding_type: MultibindingType,
}

#[derive(Debug, Clone)]
pub struct VecNode {
    pub type_: TypeData,
    pub bindings: Vec<VecBinding>,
}

impl VecNode {
    pub fn new(type_data: &TypeData) -> Box<VecNode> {
        Box::new(VecNode {
            type_: vec_type(type_data),
            bindings: vec![],
        })
    }

    pub fn add_binding(
        &mut self,
        type_data: &TypeData,
        multibinding_type: &MultibindingType,
    ) -> &mut Self {
        self.bindings.push(VecBinding {
            type_data: type_data.clone(),
            multibinding_type: multibinding_type.clone(),
        });
        self
    }
}

fn vec_type(type_data: &TypeData) -> TypeData {
    let mut vec_type = TypeData::new();
    vec_type.root = TypeRoot::GLOBAL;
    vec_type.path = "std::vec::Vec".to_string();
    vec_type.args.push(type_data.clone());
    vec_type.qualifier = type_data.qualifier.clone();
    vec_type
}

impl Node for VecNode {
    fn get_name(&self) -> String {
        return format!("{} (multibinding)", self.type_.readable());
    }

    fn generate_implementation(&self, _graph: &Graph) -> Result<ComponentSections, TokenStream> {
        let name_ident = self.get_identifier();
        let provides_type = self.type_.syn_type();
        let mut into_vecs = quote! {};
        let mut elements_into_vecs = quote! {};
        for dependency in &self.bindings {
            match dependency.multibinding_type {
                MultibindingType::IntoVec => {
                    let ident = dependency.type_data.identifier();
                    into_vecs = quote! {#into_vecs self.#ident(),}
                }
                MultibindingType::ElementsIntoVec => {
                    let ident = dependency.type_data.identifier();
                    elements_into_vecs = quote! {
                        #elements_into_vecs
                        result.extend(self.#ident());
                    }
                }
                _ => {}
            }
        }

        let mut result = ComponentSections::new();
        result.add_methods(quote! {
            #[allow(unused_mut)]
            #[allow(dead_code)]
            fn #name_ident(&'_ self) -> #provides_type{
                let mut result = vec![#into_vecs];
                #elements_into_vecs;
                result
            }
        });

        Ok(result)
    }

    fn merge(&self, new_node: &dyn Node) -> Result<Box<dyn Node>, TokenStream> {
        if new_node.type_id() != TypeId::of::<VecNode>() {
            return <dyn Node>::duplicated(self, new_node);
        }
        let vec_node = new_node.as_any().downcast_ref::<VecNode>().unwrap();
        let mut new_vec = self.bindings.clone();
        new_vec.extend(vec_node.bindings.iter().cloned());
        Ok(Box::new(VecNode {
            type_: self.type_.clone(),
            bindings: new_vec,
        }))
    }

    fn get_type(&self) -> &TypeData {
        &self.type_
    }

    fn get_identifier(&self) -> Ident {
        self.type_.identifier()
    }

    fn get_dependencies(&self) -> Vec<DependencyData> {
        self.bindings
            .iter()
            .map(|binding| DependencyData::from_type(&binding.type_data))
            .collect()
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
