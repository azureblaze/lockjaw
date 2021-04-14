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
use crate::nodes::node::Node;
use crate::type_data::TypeData;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug)]
pub struct ProviderNode {
    pub type_: TypeData,
    pub dependencies: Vec<TypeData>,
    pub node: Box<dyn Node>,
}

impl ProviderNode {
    pub fn new(node: &dyn Node) -> ProviderNode {
        ProviderNode {
            type_: <dyn Node>::provider_type(node.get_type()),
            dependencies: vec![node.get_type().clone()],
            node: node.clone_box(),
        }
    }
}

impl Clone for ProviderNode {
    fn clone(&self) -> Self {
        ProviderNode {
            type_: self.type_.clone(),
            dependencies: self.dependencies.clone(),
            node: self.node.clone_box(),
        }
    }
}

impl Node for ProviderNode {
    fn get_name(&self) -> String {
        return format!("Provider<{}>", self.dependencies[0].readable());
    }

    fn generate_provider(&self, _graph: &Graph) -> Result<ComponentSections, TokenStream> {
        let arg_provider_name = self.node.get_type().identifier();
        let name_ident = self.get_identifier();
        let provides_type = self.node.get_type().syn_type();

        let mut result = ComponentSections::new();

        result.add_methods(quote! {
            fn #name_ident(&'_ self) -> lockjaw::Provider<'_, #provides_type>{
                lockjaw::Provider::new(move || self.#arg_provider_name())
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
        false
    }

    fn set_scoped(&mut self, _scoped: bool) {
        panic!("should not set scoped on Provider<>");
    }

    fn clone_box(&self) -> Box<dyn Node> {
        return Box::new(self.clone());
    }
}
