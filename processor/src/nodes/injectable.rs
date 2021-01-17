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
                .fields
                .iter()
                .filter(|field| field.get_injected())
                .map(|field| field.get_field_type().clone())
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
        let mut params = quote! {};
        for field in self.injectable.get_fields() {
            if field.get_injected() {
                let param_name = format_ident!("{}", field.get_name());
                let param_type = field.get_field_type().syn_type();
                if field.get_field_type().get_field_ref() {
                    params = quote! {
                       #params #param_name : &'a #param_type,
                    };
                } else {
                    params = quote! {
                       #params #param_name : #param_type,
                    }
                }
            }
        }
        let mut ctor_params = quote! {};
        for field in &self.injectable.fields {
            let param_name = format_ident!("{}", field.get_name());
            if field.get_injected() {
                let param_provider_name = field.get_field_type().identifier();
                ctor_params = quote! {
                   #ctor_params
                   #param_name : self.#param_provider_name(),
                }
            } else {
                let param_type = field.get_field_type().syn_type();
                ctor_params = quote! {
                   #ctor_params
                   #param_name : <#param_type>::default(),
                }
            }
        }
        let lifetime;
        if has_ref {
            lifetime = quote! {<'_>};
        } else {
            lifetime = quote! {};
        }

        let name_ident = self.type_.identifier();
        let injectable_path = self.type_.syn_type();
        let mut result = ComponentSections::new();
        result.add_methods(quote! {
            fn #name_ident(&'_ self) -> #injectable_path #lifetime{
                #injectable_path{#ctor_params}
            }
        });
        Ok(result)
    }

    fn merge(&self, new_node: &dyn Node) -> Result<Box<dyn Node>, TokenStream> {
        Node::duplicated(self, new_node)
    }

    fn can_depend(
        &self,
        target_node: &dyn Node,
        ancestors: &Vec<String>,
    ) -> Result<(), TokenStream> {
        if target_node.is_scoped() {
            return Node::no_scope(target_node, ancestors);
        }
        Ok(())
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
