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
use std::fmt::Debug;

use proc_macro2::{Ident, TokenStream};
use quote::format_ident;

use crate::error::compile_error;
use crate::graph::{ComponentSections, Graph};
use crate::nodes::boxed::BoxedNode;
use crate::nodes::component_lifetime::ComponentLifetimeNode;
use crate::nodes::lazy::LazyNode;
use crate::nodes::provider::ProviderNode;
use crate::nodes::scoped::ScopedNode;
use crate::type_data::ProcessorTypeData;
use lockjaw_common::manifest::{BuilderModules, TypeRoot};
use lockjaw_common::type_data::TypeData;
use std::any::Any;
use std::cell::Cell;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct DependencyData {
    pub type_: TypeData,
    pub message: String,
}

impl DependencyData {
    pub fn from_type(type_: &TypeData) -> Self {
        DependencyData {
            type_: type_.clone(),
            message: String::new(),
        }
    }

    pub fn from_type_vec(type_vec: &Vec<TypeData>) -> Vec<Self> {
        type_vec.iter().map(Self::from_type).collect()
    }
}

pub trait Node: Debug + Any {
    fn get_name(&self) -> String;
    fn generate_implementation(&self, graph: &Graph) -> Result<ComponentSections, TokenStream>;
    fn merge(&self, new_node: &dyn Node) -> Result<Box<dyn Node>, TokenStream> {
        <dyn Node>::duplicated_impl(
            &self.get_type().canonical_string_path(),
            &self.get_name(),
            &new_node.get_name(),
        )
    }
    fn can_depend(
        &self,
        target_node: &dyn Node,
        ancestors: &Vec<String>,
    ) -> Result<(), TokenStream> {
        if !target_node.get_type().scopes.is_empty() {
            return <dyn Node>::no_scope(target_node, ancestors);
        }
        Ok(())
    }

    fn get_type(&self) -> &TypeData;
    fn get_identifier(&self) -> Ident {
        self.get_type().identifier()
    }
    fn get_dependencies(&self) -> Vec<DependencyData> {
        Vec::new()
    }
    fn get_optional_dependencies(&self) -> Vec<TypeData> {
        Vec::new()
    }

    fn is_runtime_dependency(&self) -> bool {
        false
    }

    fn clone_box(&self) -> Box<dyn Node>;
    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;
}

impl dyn Node {
    pub fn duplicated<T>(node: &dyn Node, new_node: &dyn Node) -> Result<T, TokenStream> {
        <dyn Node>::duplicated_impl(
            &node.get_type().canonical_string_path(),
            &node.get_name(),
            &new_node.get_name(),
        )
    }

    fn duplicated_impl<T>(path: &str, name: &str, other_name: &str) -> Result<T, TokenStream> {
        return compile_error(&format!(
            "found duplicated bindings for {}, provided by:\n\t{}\n\t{}",
            path, name, other_name
        ));
    }

    pub fn no_scope(target_node: &dyn Node, ancestors: &Vec<String>) -> Result<(), TokenStream> {
        let mut reverse_ancestors = ancestors.clone();
        reverse_ancestors.reverse();
        compile_error(&format!(
            "unable to provide scoped binding as regular type {}\nrequested by:{}",
            target_node.get_name(),
            reverse_ancestors.join("\nrequested by:")
        ))
    }

    pub fn generate_node(
        map: &HashMap<Ident, Box<dyn Node>>,
        dependency: &TypeData,
    ) -> Option<Box<dyn Node>> {
        if dependency.field_ref {
            return Some(ScopedNode::for_type(dependency));
        }
        if dependency.root != TypeRoot::GLOBAL {
            return None;
        }
        let path = format!("{}::{}", dependency.field_crate, dependency.path);
        match path.as_str() {
            "::std::boxed::Box" => BoxedNode::for_type(dependency),
            "::lockjaw::Provider" => ProviderNode::for_type(dependency),
            "::lockjaw::Lazy" => LazyNode::for_type(dependency),
            "::lockjaw::Cl" => ComponentLifetimeNode::for_type(map, dependency),
            _ => None,
        }
    }

    pub fn get_module_instance(
        manifest: &BuilderModules,
        module_type: &TypeData,
    ) -> ModuleInstance {
        let ident = module_type.identifier();

        for module in &manifest.builder_modules {
            if module.type_data.identifier().eq(&ident) {
                return ModuleInstance {
                    type_: module_type.clone(),
                    name: format_ident!("{}", module.name.to_owned()),
                };
            }
        }

        return ModuleInstance {
            type_: module_type.clone(),
            name: module_type.identifier(),
        };
    }
}

/// An item in a module
#[derive(Debug, Clone)]
pub struct ModuleInstance {
    pub type_: TypeData,
    pub name: syn::Ident,
}

thread_local! {
    static MULTIBINDING_ID : Cell<i32> = Cell::new(0);
}

pub fn get_multibinding_id() -> i32 {
    MULTIBINDING_ID.with(|m| {
        let id = m.get();
        m.set(id + 1);
        id
    })
}
