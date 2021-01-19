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
use crate::protos::manifest::Type;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
#[derive(Debug, Clone)]
pub struct InjectableNode {
    pub type_: Type,
    pub dependencies: Vec<Type>,
    pub scoped: bool,

    pub injectable: crate::protos::manifest::Injectable,
}

impl InjectableNode {
    pub fn new(injectable: &crate::protos::manifest::Injectable) -> Vec<Box<dyn Node>> {
        let node = Box::new(InjectableNode {
            type_: injectable.get_field_type().clone(),
            dependencies: injectable
                .get_dependencies()
                .iter()
                .map(|dep| dep.get_field_type().clone())
                .collect(),
            scoped: false,
            injectable: injectable.clone(),
        });
        Node::generate_node_variants(node)
    }
}

impl Node for InjectableNode {
    fn get_name(&self) -> String {
        format!("{} (injectable)", self.type_.canonical_string_path())
    }

    fn generate_provider(&self, graph: &Graph) -> Result<ComponentSections, TokenStream> {
        let has_ref = graph.has_scoped_deps(self)?;
        let mut ctor_params = quote! {};
        for dependency in self.injectable.get_dependencies() {
            let param_provider_name = dependency.get_field_type().identifier();
            ctor_params = quote! {
               #ctor_params
               self.#param_provider_name(),
            }
        }

        let lifetime;
        if has_ref {
            lifetime = quote! {<'_>};
        } else {
            lifetime = quote! {};
        }

        let name_ident = self.get_identifier();
        let injectable_path = self.type_.syn_type();
        let ctor_name = format_ident!("{}", self.injectable.get_ctor_name());
        let mut result = ComponentSections::new();
        result.add_methods(quote! {
            fn #name_ident(&'_ self) -> #injectable_path #lifetime{
                #injectable_path::#ctor_name(#ctor_params)
            }
        });
        Ok(result)
    }

    fn get_type(&self) -> &Type {
        &self.type_
    }

    fn get_dependencies(&self) -> &Vec<Type> {
        &self.dependencies
    }

    fn is_scoped(&self) -> bool {
        self.scoped
    }

    fn set_scoped(&mut self, scoped: bool) {
        self.scoped = scoped;
    }

    fn clone_box(&self) -> Box<dyn Node> {
        Box::new(self.clone())
    }
}
