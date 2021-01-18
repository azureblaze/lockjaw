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

use crate::graph::{ComponentSections, Graph};
use crate::nodes::node::Node;
use crate::protos::manifest::{Component, Dependency, Type};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

#[derive(Debug, Clone)]
pub struct ProvisionNode {
    dependency: Dependency,
    component: Component,
    dependencies: Vec<Type>,
}

impl ProvisionNode {
    pub fn new(dependency: Dependency, component: Component) -> Self {
        ProvisionNode {
            dependencies: vec![dependency.get_field_type().clone()],
            dependency,
            component,
        }
    }
}

impl Node for ProvisionNode {
    fn get_name(&self) -> String {
        format!(
            "{}.{}",
            self.component.get_field_type().canonical_string_path(),
            self.dependency.get_name()
        )
    }

    fn generate_provider(&self, _graph: &Graph) -> Result<ComponentSections, TokenStream> {
        let mut result = ComponentSections::new();
        let dependency_name = self.get_identifier();
        let dependency_path = self.dependency.get_field_type().syn_type();
        let dependency_type;
        if self.dependency.get_field_type().get_field_ref() {
            dependency_type = quote! {& #dependency_path};
        } else {
            dependency_type = quote! {#dependency_path}
        }
        let provider_name = self.dependency.get_field_type().identifier();
        result.add_trait_methods(quote! {
           fn #dependency_name(&self) -> #dependency_type {
              self.#provider_name()
           }
        });
        Ok(result)
    }

    fn get_type(&self) -> &Type {
        unimplemented!()
    }

    fn get_identifier(&self) -> Ident {
        format_ident!("{}", self.dependency.get_name())
    }

    fn get_dependencies(&self) -> &Vec<Type> {
        &self.dependencies
    }

    fn is_scoped(&self) -> bool {
        unimplemented!()
    }

    fn set_scoped(&mut self, _scoped: bool) {
        unimplemented!()
    }

    fn clone_box(&self) -> Box<dyn Node> {
        Box::new(self.clone())
    }
}
