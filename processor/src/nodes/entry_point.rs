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

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use crate::graph::{ComponentSections, Graph};
use crate::manifest::{ComponentType, EntryPoint};
use crate::nodes::node::{DependencyData, Node};
use crate::type_data::TypeData;
use crate::{component_visibles, entrypoints};
use std::any::Any;

#[derive(Debug, Clone)]
pub struct EntryPointNode {
    dependencies: Vec<TypeData>,
    entry_point: EntryPoint,
}

impl EntryPointNode {
    pub fn new(entry_point: &EntryPoint) -> Self {
        EntryPointNode {
            dependencies: entry_point
                .provisions
                .iter()
                .map(|dep| dep.type_data.clone())
                .collect(),
            entry_point: entry_point.clone(),
        }
    }
}

impl Node for EntryPointNode {
    fn get_name(&self) -> String {
        format!(
            "{} (Entry point installed in {})",
            self.entry_point.type_data.readable(),
            self.entry_point.component.readable()
        )
    }

    fn generate_implementation(&self, graph: &Graph) -> Result<ComponentSections, TokenStream> {
        let mut result = ComponentSections::new();

        let mut provisions = quote! {};
        for provision in &self.entry_point.provisions {
            let dependency_name = format_ident!("{}", provision.name);
            let dependency_path =
                component_visibles::visible_type(graph.manifest, &provision.type_data).syn_type();
            let provider_name = provision.type_data.identifier();
            provisions = quote! {
                #provisions
               fn #dependency_name(&self) -> #dependency_path {
                  self.#provider_name()
               }
            }
        }

        let entry_point_syn_type =
            component_visibles::visible_type(graph.manifest, &self.entry_point.type_data)
                .syn_type();

        let getter_name = entrypoints::getter_name(&self.entry_point);

        let component_impl_name = graph.component.impl_ident();

        let lifetime = if graph.component.component_type == ComponentType::Subcomponent {
            quote! {<'a>}
        } else {
            quote! {}
        };
        let component_name =
            component_visibles::visible_type(graph.manifest, &graph.component.type_data).syn_type();

        result.add_items(quote! {
            impl #lifetime #entry_point_syn_type for #component_impl_name #lifetime {
                #provisions
            }

            #[no_mangle]
            #[allow(non_snake_case)]
            fn #getter_name<'a>(component: &'a dyn #component_name) -> &'a dyn #entry_point_syn_type {
                unsafe {
                    &*(component as *const dyn #component_name
                        as *const #component_impl_name
                        as *const dyn #entry_point_syn_type)
                }
            }
        });

        Ok(result)
    }

    fn get_type(&self) -> &TypeData {
        unimplemented!()
    }

    fn get_identifier(&self) -> Ident {
        format_ident!("{}", self.entry_point.type_data.identifier())
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
