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

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use crate::graph::{ComponentSections, Graph};
use crate::manifest::{Component, Dependency};
use crate::nodes::node::{DependencyData, Node};
use crate::type_data::TypeData;
use std::any::Any;

#[derive(Debug, Clone)]
pub struct ProvisionNode {
    dependency: Dependency,
    component: Component,
    dependencies: Vec<TypeData>,
}

impl ProvisionNode {
    pub fn new(dependency: Dependency, component: Component) -> Self {
        ProvisionNode {
            dependencies: vec![dependency.type_data.clone()],
            dependency,
            component,
        }
    }
}

impl Node for ProvisionNode {
    fn get_name(&self) -> String {
        format!(
            "{}.{}",
            self.component.type_data.canonical_string_path(),
            self.dependency.name
        )
    }

    fn generate_implementation(&self, _graph: &Graph) -> Result<ComponentSections, TokenStream> {
        let mut result = ComponentSections::new();
        let dependency_name = self.get_identifier();
        let dependency_path = self.dependency.type_data.syn_type();
        let provider_name = self.dependency.type_data.identifier();
        result.add_trait_methods(quote! {
           fn #dependency_name(&self) -> #dependency_path {
              self.#provider_name()
           }
        });
        Ok(result)
    }

    fn get_type(&self) -> &TypeData {
        unimplemented!()
    }

    fn get_identifier(&self) -> Ident {
        format_ident!("{}", self.dependency.name)
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
