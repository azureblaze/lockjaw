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
use crate::graph::{ComponentSections, Graph};
use crate::manifest::TypeRoot;
use crate::nodes::node::{DependencyData, Node};
use crate::type_data::TypeData;
use proc_macro2::TokenStream;
use quote::quote;
use std::any::Any;

#[derive(Debug)]
pub struct ProviderNode {
    pub type_: TypeData,
    pub dependencies: Vec<TypeData>,
    pub inner: TypeData,
}

impl ProviderNode {
    pub fn for_type(type_: &TypeData) -> Option<Box<dyn Node>> {
        let inner = type_.args.get(0).unwrap();
        Some(Box::new(ProviderNode {
            type_: ProviderNode::provider_type(inner),
            dependencies: vec![inner.clone()],
            inner: inner.clone(),
        }))
    }

    pub fn provider_type(type_: &TypeData) -> TypeData {
        let mut provider_type = TypeData::new();
        provider_type.root = TypeRoot::GLOBAL;
        provider_type.path = "lockjaw::Provider".to_string();
        provider_type.args.push(type_.clone());

        provider_type
    }
}

impl Clone for ProviderNode {
    fn clone(&self) -> Self {
        ProviderNode {
            type_: self.type_.clone(),
            dependencies: self.dependencies.clone(),
            inner: self.inner.clone(),
        }
    }
}

impl Node for ProviderNode {
    fn get_name(&self) -> String {
        return format!("Provider<{}>", self.dependencies[0].readable());
    }

    fn generate_implementation(&self, graph: &Graph) -> Result<ComponentSections, TokenStream> {
        let arg_provider_name = self.inner.identifier();
        let name_ident = self.get_identifier();
        let provides_type =
            component_visibles::visible_type(graph.manifest, &self.inner).syn_type();

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

    fn get_dependencies(&self) -> Vec<DependencyData> {
        DependencyData::from_type_vec(&self.dependencies)
    }
    fn is_runtime_dependency(&self) -> bool {
        true
    }

    fn clone_box(&self) -> Box<dyn Node> {
        return Box::new(self.clone());
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}
