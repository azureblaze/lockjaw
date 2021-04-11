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

use crate::graph::ComponentSections;
use crate::graph::Graph;
use crate::manifest::Provider;
use crate::nodes::node::{ModuleInstance, Node};
use crate::type_data::TypeData;

#[derive(Debug, Clone)]
pub struct BindsNode {
    pub type_: TypeData,
    pub dependencies: Vec<TypeData>,
    pub scoped: bool,

    pub module_instance: ModuleInstance,
    pub provider: Provider,
}

impl Node for BindsNode {
    fn get_name(&self) -> String {
        format!(
            "{}.{} (module binds)",
            self.module_instance.type_.canonical_string_path(),
            self.provider.name
        )
    }

    fn generate_provider(&self, _graph: &Graph) -> Result<ComponentSections, TokenStream> {
        let arg = self
            .provider
            .dependencies
            .first()
            .expect("binds must have one arg");
        let arg_provider_name = arg.type_data.identifier();

        let name_ident = self.get_identifier();
        let type_path = self.type_.syn_type();

        let mut result = ComponentSections::new();
        if arg.type_data.field_ref {
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
