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

use crate::component_visibles;
use crate::graph::{ComponentSections, Graph};
use crate::nodes::node::{DependencyData, Node};
use crate::type_data::TypeData;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::any::Any;

#[derive(Debug, Clone)]
pub struct InjectableNode {
    pub type_: TypeData,
    pub dependencies: Vec<TypeData>,
    pub scoped: bool,

    pub injectable: crate::manifest::Injectable,
}

impl InjectableNode {
    pub fn new(injectable: &crate::manifest::Injectable) -> Box<dyn Node> {
        let type_ = if injectable.container.is_some() {
            let mut container = injectable.container.as_ref().unwrap().clone();
            container.args.push(injectable.type_data.clone());
            container
        } else {
            injectable.type_data.clone()
        };
        Box::new(InjectableNode {
            type_,
            dependencies: injectable
                .dependencies
                .iter()
                .map(|dep| dep.type_data.clone())
                .collect(),
            scoped: false,
            injectable: injectable.clone(),
        })
    }
}

impl Node for InjectableNode {
    fn get_name(&self) -> String {
        format!("{} (injectable)", self.type_.canonical_string_path())
    }

    fn generate_implementation(&self, graph: &Graph) -> Result<ComponentSections, TokenStream> {
        let has_ref = graph.has_scoped_deps(&self.type_.identifier())?;
        let mut ctor_params = quote! {};
        for dependency in &self.injectable.dependencies {
            let param_provider_name = dependency.type_data.identifier();
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
        let injectable_path =
            component_visibles::visible_type(graph.manifest, &self.injectable.type_data).syn_type();
        let ctor_name = format_ident!("{}", self.injectable.ctor_name);
        let mut result = ComponentSections::new();
        if self.injectable.container.is_some() {
            let mut container = self.injectable.container.as_ref().unwrap().clone();
            container.args.push(component_visibles::visible_type(
                graph.manifest,
                &self.injectable.type_data,
            ));
            let result_path = container.syn_type();
            let container_type = self.injectable.container.as_ref().unwrap().syn_type();
            result.add_methods(quote! {
                fn #name_ident(&'_ self) -> #result_path #lifetime{
                    #container_type::new(#injectable_path::#ctor_name(#ctor_params))
                }
            });
        } else {
            result.add_methods(quote! {
                fn #name_ident(&'_ self) -> #injectable_path #lifetime{
                    #injectable_path::#ctor_name(#ctor_params)
                }
            });
        }
        Ok(result)
    }

    fn get_type(&self) -> &TypeData {
        &self.type_
    }

    fn get_dependencies(&self) -> Vec<DependencyData> {
        DependencyData::from_type_vec(&self.dependencies)
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
