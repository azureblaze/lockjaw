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
use crate::manifest::TypeRoot;
use crate::nodes::node::Node;
use crate::type_data::TypeData;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::any::TypeId;
use std::iter::Extend;

#[derive(Debug, Clone)]
pub struct VecNode {
    pub type_: TypeData,
    pub dependencies: Vec<TypeData>,
}

impl VecNode {
    pub fn new(type_data: &TypeData) -> Box<VecNode> {
        Box::new(VecNode {
            type_: vec_type(type_data),
            dependencies: vec![],
        })
    }
}

fn vec_type(type_data: &TypeData) -> TypeData {
    let mut vec_type = TypeData::new();
    vec_type.root = TypeRoot::GLOBAL;
    vec_type.path = "std::vec::Vec".to_string();
    vec_type.args.push(type_data.clone());
    vec_type
}

impl Node for VecNode {
    fn get_name(&self) -> String {
        return format!("{} (multibinding)", self.type_.readable());
    }

    fn generate_implementation(&self, _graph: &Graph) -> Result<ComponentSections, TokenStream> {
        let name_ident = self.get_identifier();
        let provides_type = self.type_.syn_type();

        let mut items = quote! {};
        for dependency in &self.dependencies {
            let ident = dependency.identifier();
            items = quote! {#items self.#ident(),}
        }

        let mut result = ComponentSections::new();
        result.add_methods(quote! {
            fn #name_ident(&'_ self) -> #provides_type{
                vec![#items]
            }
        });

        Ok(result)
    }

    fn merge(&self, new_node: &dyn Node) -> Result<Box<dyn Node>, TokenStream> {
        if new_node.type_id() != TypeId::of::<VecNode>() {
            return <dyn Node>::duplicated(self, new_node);
        }
        let mut new_vec = self.dependencies.clone();
        new_vec.extend(new_node.get_dependencies().iter().cloned());
        Ok(Box::new(VecNode {
            type_: self.type_.clone(),
            dependencies: new_vec,
        }))
    }

    fn get_type(&self) -> &TypeData {
        &self.type_
    }

    fn get_identifier(&self) -> Ident {
        self.type_.identifier()
    }

    fn get_dependencies(&self) -> &Vec<TypeData> {
        &self.dependencies
    }

    fn clone_box(&self) -> Box<dyn Node> {
        Box::new(self.clone())
    }
}
