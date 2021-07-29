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

use crate::component_visibles;
use crate::graph::{ComponentSections, Graph, MissingDependency};
use crate::manifest::MultibindingType;
use crate::nodes::node::Node;
use crate::type_data::TypeData;
use proc_macro2::TokenStream;
use quote::quote;
use std::any::Any;

#[derive(Debug, Clone)]
pub struct ParentNode {
    pub type_: TypeData,
    pub parent_type: TypeData,
}

impl ParentNode {
    pub fn new(parent_dep: &MissingDependency) -> Result<Box<Self>, TokenStream> {
        let mut type_ = parent_dep.type_data.clone();
        if parent_dep.multibinding_type != MultibindingType::None {
            type_.identifier_suffix.push_str("_parent");
        }
        Ok(Box::new(ParentNode {
            type_,
            parent_type: parent_dep.type_data.clone(),
        }))
    }
}

impl Node for ParentNode {
    fn get_name(&self) -> String {
        format!("{} (parent component access)", self.type_.readable())
    }

    fn generate_implementation(&self, graph: &Graph) -> Result<ComponentSections, TokenStream> {
        let name_ident = self.get_identifier();
        let parent_ident = self.parent_type.identifier();
        let syn_type = component_visibles::visible_type(graph.manifest, &self.type_).syn_type();

        let mut result = ComponentSections::new();

        result.add_methods(quote! {
            fn #name_ident(&'_ self) -> #syn_type{
                self.parent.#parent_ident()
            }
        });

        Ok(result)
    }

    fn get_type(&self) -> &TypeData {
        &self.type_
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
