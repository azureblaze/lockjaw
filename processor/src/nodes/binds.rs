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
use crate::nodes::node::{ModuleInstance, Node};
use crate::protos::manifest::{Provider, Type};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub struct BindsNode {
    pub type_: Type,
    pub dependencies: Vec<Type>,
    pub scoped: bool,

    pub module_instance: ModuleInstance,
    pub provider: Provider,
}

impl Node for BindsNode {
    fn get_name(&self) -> String {
        format!(
            "{}.{} (module binds)",
            self.module_instance.type_.canonical_string_path(),
            self.provider.get_name()
        )
    }

    fn generate_provider(&self, _graph: &Graph) -> Result<ComponentSections, TokenStream> {
        let arg = self
            .provider
            .get_dependencies()
            .first()
            .expect("binds must have one arg");
        let arg_provider_name = arg.get_field_type().identifier();

        let name_ident = self.type_.identifier();
        let type_path = self.type_.syn_type();

        let mut result = ComponentSections::new();
        if arg.get_field_type().get_field_ref() {
            result.add_methods(quote! {
                fn #name_ident(&'_ self) -> #type_path{
                    lockjaw::MaybeScoped::Ref(self.#arg_provider_name())
                }
            });
        } else {
            result.add_methods(quote! {
                fn #name_ident(&'_ self) -> #type_path{
                    lockjaw::MaybeScoped::Val(Box::new(self.#arg_provider_name()))
                }
            });
        }
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
