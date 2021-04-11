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
use crate::graph::ComponentSections;
use crate::graph::Graph;
use crate::manifest::Type;
use crate::nodes::node::Node;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

#[derive(Debug)]
pub struct ScopedNode {
    pub type_: Type,
    pub dependencies: Vec<Type>,
    pub scoped: bool,

    pub node: Box<dyn Node>,
}

impl Clone for ScopedNode {
    fn clone(&self) -> Self {
        return ScopedNode {
            type_: self.type_.clone(),
            dependencies: self.dependencies.clone(),
            scoped: self.scoped.clone(),
            node: self.node.clone_box(),
        };
    }
}

impl Node for ScopedNode {
    fn get_name(&self) -> String {
        format!("ref {}", self.type_.canonical_string_path())
    }

    fn generate_provider(&self, _graph: &Graph) -> Result<ComponentSections, TokenStream> {
        let arg_provider_name = self.node.get_type().identifier();
        let once_name = format_ident!("once_{}", self.type_.identifier());
        let once_type = self.node.get_type().syn_type();

        let name_ident = self.get_identifier();
        let type_path = self.type_.syn_type();
        let mut result = ComponentSections::new();
        result.add_fields(quote! {
            #once_name : lockjaw::Once<#once_type>,
        });
        result.add_ctor_params(quote! {#once_name : lockjaw::Once::new(),});
        result.add_methods(quote! {
            fn #name_ident(&'_ self) -> &#type_path{
                self.#once_name.get(|| self.#arg_provider_name())
            }
        });
        Ok(result)
    }

    fn can_depend(
        &self,
        _target_node: &dyn Node,
        _ancestors: &Vec<String>,
    ) -> Result<(), TokenStream> {
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
