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
use crate::graph::ComponentSections;
use crate::graph::Graph;
use crate::nodes::node::{DependencyData, Node};
use crate::type_data::TypeData;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::any::Any;

#[derive(Debug)]
pub struct ScopedNode {
    pub type_: TypeData,
    pub dependencies: Vec<TypeData>,
    pub target: TypeData,
}

impl ScopedNode {
    pub fn for_type(type_: &TypeData) -> Box<dyn Node> {
        let mut non_ref = type_.clone();

        non_ref.field_ref = false;
        return Box::new(ScopedNode {
            type_: type_.clone(),
            dependencies: vec![non_ref.clone()],
            target: non_ref.clone(),
        });
    }
}

impl Clone for ScopedNode {
    fn clone(&self) -> Self {
        return ScopedNode {
            type_: self.type_.clone(),
            dependencies: self.dependencies.clone(),
            target: self.target.clone(),
        };
    }
}

impl Node for ScopedNode {
    fn get_name(&self) -> String {
        format!("ref {}", self.type_.canonical_string_path())
    }

    fn generate_implementation(&self, graph: &Graph) -> Result<ComponentSections, TokenStream> {
        let arg_provider_name = self.target.identifier();
        let once_name = format_ident!("once_{}", self.type_.identifier());
        let name_ident = self.get_identifier();
        let type_path =
            component_visibles::visible_ref_type(graph.manifest, &self.type_).syn_type();
        let mut result = ComponentSections::new();
        let once_inner_type =
            if !self.target.args.is_empty() && graph.has_lifetime(&self.target.args[0]) {
                let mut container = self.target.clone();
                container.args = Vec::new();
                let container_type = container.syn_type();
                let target_type = self.target.args[0].syn_type();
                quote! {
                    #container_type<#target_type<'static>>
                }
            } else {
                let lifetime = if graph.has_lifetime(&self.target) {
                    //  effectively component lifetime since the component owns it.
                    quote! {<'static>}
                } else {
                    quote! {}
                };
                let once_type =
                    component_visibles::visible_type(graph.manifest, &self.target).syn_type();
                quote! {#once_type #lifetime}
            };
        result.add_fields(quote! {
            #once_name : lockjaw::Once<#once_inner_type>,
        });
        result.add_ctor_params(quote! {#once_name : lockjaw::Once::new(),});

        let component_name = graph.component.impl_ident();
        result.add_methods(quote! {
            fn #name_ident(&'_ self) -> #type_path{
                // prevent self from being borrowed into once, which has 'static lifetime, but in
                // practice limited to the component's lifetime.
                // safe since lambda in Once.get() is invoked immediately.
                unsafe{
                    let this: *const #component_name = ::std::mem::transmute(self);
                    let result = self.#once_name.get(|| (&*this).#arg_provider_name());
                    // erases the 'static lifetime on Once, and reassign it back to '_ (the component's lifetime)
                    std::mem::transmute(result)
                }
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
