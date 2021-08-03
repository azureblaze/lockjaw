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
use crate::manifest::{Binding, TypeRoot};
use crate::nodes::node::Node;
use crate::type_data::TypeData;
use std::any::Any;

#[derive(Debug, Clone)]
pub struct BindsOptionOfNode {
    pub type_: TypeData,
    pub dependencies: Vec<TypeData>,
    pub inner: TypeData,
}

impl BindsOptionOfNode {
    pub fn new(binding: &Binding) -> Vec<Box<dyn Node>> {
        vec![Box::new(BindsOptionOfNode {
            type_: BindsOptionOfNode::option_type(&binding.type_data),
            dependencies: vec![binding.type_data.clone()],
            inner: binding.type_data.clone(),
        })]
    }

    pub fn option_type(type_: &TypeData) -> TypeData {
        let mut option_type = TypeData::new();
        option_type.root = TypeRoot::GLOBAL;
        option_type.path = "std::option::Option".to_string();
        option_type.args.push(type_.clone());
        option_type
    }
}

impl Node for BindsOptionOfNode {
    fn get_name(&self) -> String {
        format!("Option<{}> (binds_option_of)", self.inner.readable(),)
    }

    fn generate_implementation(&self, graph: &Graph) -> Result<ComponentSections, TokenStream> {
        let inner_provider_name = self.inner.identifier();

        let name_ident = self.get_identifier();
        let type_path =
            component_visibles::visible_nested_type(graph.manifest, &self.type_).syn_type();
        let body;
        if graph.has_node(&self.inner) {
            body = quote! { Option::Some(self.#inner_provider_name()) }
        } else {
            body = quote! { Option::None }
        }

        let mut result = ComponentSections::new();
        result.add_methods(quote! {
            fn #name_ident(&'_ self) -> #type_path{
                #body
            }
        });
        Ok(result)
    }

    fn get_type(&self) -> &TypeData {
        &self.type_
    }

    fn get_optional_dependencies(&self) -> Vec<TypeData> {
        self.dependencies.clone()
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
