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
use crate::nodes::node::Node;
use crate::type_data::TypeData;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug)]
pub struct ComponentLifetimeNode {
    pub type_: TypeData,
    pub dependencies: Vec<TypeData>,
    pub scoped: bool,

    pub node: Box<dyn Node>,
}
impl Clone for ComponentLifetimeNode {
    fn clone(&self) -> Self {
        ComponentLifetimeNode {
            type_: self.type_.clone(),
            dependencies: self.dependencies.clone(),
            scoped: self.scoped.clone(),
            node: self.node.clone_box(),
        }
    }
}

impl Node for ComponentLifetimeNode {
    fn get_name(&self) -> String {
        format!("{} (auto boxed)", self.type_.canonical_string_path())
    }

    fn generate_provider(&self, _graph: &Graph) -> Result<ComponentSections, TokenStream> {
        let arg_provider_name = self.node.get_type().identifier();
        let name_ident = self.get_identifier();
        let type_path = self.type_.syn_type();

        let mut result = ComponentSections::new();
        if self.node.get_type().field_ref {
            result.add_methods(quote! {
                fn #name_ident(&'_ self) -> #type_path{
                    lockjaw::ComponentLifetime::Ref(self.#arg_provider_name())
                }
            });
        } else {
            result.add_methods(quote! {
                fn #name_ident(&'_ self) -> #type_path{
                    lockjaw::ComponentLifetime::Val(Box::new(self.#arg_provider_name()))
                }
            });
        }
        Ok(result)
    }

    fn merge(&self, new_node: &dyn Node) -> Result<Box<dyn Node>, TokenStream> {
        if self
            .type_
            .canonical_string_path()
            .eq(&new_node.get_type().canonical_string_path())
        {
            return Ok(self.clone_box());
        }
        <dyn Node>::duplicated(self, new_node)
    }

    fn can_depend(
        &self,
        _target_node: &dyn Node,
        _ancestors: &Vec<String>,
    ) -> Result<(), TokenStream> {
        Ok(())
    }

    fn get_type(&self) -> &TypeData {
        &self.type_
    }

    fn get_dependencies(&self) -> &Vec<TypeData> {
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
