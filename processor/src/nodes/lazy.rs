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
use crate::nodes::provider::ProviderNode;
use crate::type_data::TypeData;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug)]
pub struct LazyNode {
    pub type_: TypeData,
    pub dependencies: Vec<TypeData>,
    pub node: Box<dyn Node>,
}

impl LazyNode {
    pub fn new(node: &dyn Node) -> Self {
        let provider = ProviderNode::new(node);
        Self {
            type_: <dyn Node>::lazy_type(node.get_type()),
            dependencies: vec![provider.get_type().clone()],
            node: provider.clone_box(),
        }
    }
}

impl Clone for LazyNode {
    fn clone(&self) -> Self {
        Self {
            type_: self.type_.clone(),
            dependencies: self.dependencies.clone(),
            node: self.node.clone_box(),
        }
    }
}

impl Node for LazyNode {
    fn get_name(&self) -> String {
        return format!("Lazy<{}>", self.dependencies[0].readable());
    }

    fn generate_provider(&self, _graph: &Graph) -> Result<ComponentSections, TokenStream> {
        let arg_provider_name = self.node.get_type().identifier();
        let name_ident = self.get_identifier();
        let lazy_type = self
            .node
            .get_dependencies()
            .get(0)
            .expect("missing Provider<T> dep for Lazy<T>")
            .syn_type();

        let mut result = ComponentSections::new();

        result.add_methods(quote! {
            fn #name_ident(&'_ self) -> lockjaw::Lazy<'_, #lazy_type>{
                lockjaw::Lazy::new(self.#arg_provider_name())
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
        panic!("should not set scoped on Lazy<>");
    }

    fn clone_box(&self) -> Box<dyn Node> {
        return Box::new(self.clone());
    }
}
