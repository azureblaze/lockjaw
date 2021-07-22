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
    pub fn for_type(type_: &TypeData) -> Option<Box<dyn Node>> {
        let mut non_ref = type_.clone();

        non_ref.field_ref = false;
        return Some(Box::new(ScopedNode {
            type_: type_.clone(),
            dependencies: vec![non_ref.clone()],
            target: non_ref.clone(),
        }));
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
        let once_type = self.target.syn_type();
        let name_ident = self.get_identifier();
        let type_path = self.type_.syn_type();
        let mut result = ComponentSections::new();
        let has_ref = graph.has_scoped_deps(&self.target.identifier())?;
        let lifetime = if has_ref {
            quote! {<'static> /* effectively component lifetime */}
        } else {
            quote! {}
        };
        result.add_fields(quote! {
            #once_name : lockjaw::Once<#once_type#lifetime>,
        });
        result.add_ctor_params(quote! {#once_name : lockjaw::Once::new(),});
        result.add_methods(quote! {
            fn #name_ident(&'_ self) -> #type_path{
                let this: *const Self = self;
                // erases the 'static lifetime on Once, and reassign it to '_ (the component's lifetime)
                self.#once_name.get(|| unsafe { &*this }.#arg_provider_name())
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
