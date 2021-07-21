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
use crate::nodes::provider::ProviderNode;
use crate::type_data::TypeData;
use proc_macro2::TokenStream;
use quote::quote;
use std::any::Any;

#[derive(Debug)]
pub struct LazyNode {
    pub type_: TypeData,
    pub dependencies: Vec<TypeData>,
    pub target: TypeData,
}

impl LazyNode {
    pub fn for_type(type_: &TypeData) -> Option<Box<dyn Node>> {
        let inner = type_.args.get(0).unwrap();
        let provider = ProviderNode::provider_type(inner);
        Some(Box::new(Self {
            type_: LazyNode::lazy_type(inner),
            dependencies: vec![provider.clone()],
            target: provider.clone(),
        }))
    }

    pub fn lazy_type(type_: &TypeData) -> TypeData {
        let mut lazy_type = TypeData::new();
        lazy_type.root = TypeRoot::GLOBAL;
        lazy_type.path = "lockjaw::Lazy".to_string();
        lazy_type.args.push(type_.clone());

        lazy_type
    }
}

impl Clone for LazyNode {
    fn clone(&self) -> Self {
        Self {
            type_: self.type_.clone(),
            dependencies: self.dependencies.clone(),
            target: self.target.clone(),
        }
    }
}

impl Node for LazyNode {
    fn get_name(&self) -> String {
        return format!("Lazy<{}>", self.dependencies[0].readable());
    }

    fn generate_implementation(&self, _graph: &Graph) -> Result<ComponentSections, TokenStream> {
        let arg_provider_name = self.target.identifier();
        let name_ident = self.get_identifier();
        let lazy_type = self
            .type_
            .args
            .get(0)
            .expect("missing T dep for Lazy<T>")
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

    fn get_dependencies(&self) -> Vec<TypeData> {
        self.dependencies.clone()
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
