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
use crate::manifest::{ComponentModuleManifest, TypeRoot};
use crate::nodes::component_lifetime::ComponentLifetimeNode;
use crate::nodes::scoped::ScopedNode;
use crate::type_data::TypeData;

pub trait Node: Debug {
    fn get_name(&self) -> String;
    fn generate_provider(&self, graph: &Graph) -> Result<ComponentSections, TokenStream>;
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
        if target_node.is_scoped() {
            return <dyn Node>::no_scope(target_node, ancestors);
        }
        Ok(())
    }

    fn get_type(&self) -> &TypeData;
    fn get_identifier(&self) -> Ident {
        self.get_type().identifier()
    }
    fn get_dependencies(&self) -> &Vec<TypeData>;
    fn is_scoped(&self) -> bool;

    fn set_scoped(&mut self, scoped: bool);

    fn clone_box(&self) -> Box<dyn Node>;
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
        compile_error(&format!(
            "unable to provide scoped binding as regular type {}\nrequested by:{}",
            target_node.get_name(),
            ancestors.join("\nrequested by:")
        ))
    }

    pub fn generate_node_variants(node: Box<dyn Node>) -> Vec<Box<dyn Node>> {
        if !node.get_type().scopes.is_empty() {
            let mut private_node = node.clone_box();
            private_node.set_scoped(true);

            let scoped_node = Box::new(ScopedNode {
                type_: <dyn Node>::ref_type(&node.get_type()),
                dependencies: vec![private_node.get_type().clone()],
                scoped: false,
                node: private_node.clone_box(),
            });

            let component_lifetime_node = Box::new(ComponentLifetimeNode {
                type_: <dyn Node>::component_lifetime_type(&private_node.get_type()),
                dependencies: vec![private_node.get_type().clone()],
                scoped: false,

                node: private_node.clone_box(),
            });

            return vec![private_node, scoped_node, component_lifetime_node];
        }

        if node.get_type().scopes.is_empty() {
            if node.get_type().path.ne("lockjaw::ComponentLifetime") {
                let boxed_node = Box::new(ComponentLifetimeNode {
                    type_: <dyn Node>::component_lifetime_type(&node.get_type()),
                    dependencies: vec![node.get_type().clone()],
                    scoped: false,
                    node: node.clone_box(),
                });
                return vec![node, boxed_node];
            }
            return vec![node];
        }
        return vec![];
    }

    pub fn get_module_instance(
        manifest: &ComponentModuleManifest,
        module_type: &TypeData,
    ) -> ModuleInstance {
        let ident = module_type.identifier();
        for module in &manifest.modules {
            if module.identifier().eq(&ident) {
                return ModuleInstance {
                    type_: module_type.clone(),
                    name: module.identifier(),
                };
            }
        }

        for module in &manifest.builder_modules {
            if module.type_data.identifier().eq(&ident) {
                return ModuleInstance {
                    type_: module_type.clone(),
                    name: format_ident!("{}", module.name.to_owned()),
                };
            }
        }

        panic!("requested module not in manifest")
    }

    pub fn component_lifetime_type(type_: &TypeData) -> TypeData {
        let mut boxed_type = TypeData::new();
        boxed_type.root = TypeRoot::GLOBAL;
        boxed_type.path = "lockjaw::ComponentLifetime".to_string();
        boxed_type.args.push(type_.clone());
        boxed_type
    }

    pub fn ref_type(type_: &TypeData) -> TypeData {
        let mut ref_type = type_.clone();
        ref_type.field_ref = true;
        ref_type
    }
}

/// An item in a module
#[derive(Debug, Clone)]
pub struct ModuleInstance {
    pub type_: TypeData,
    pub name: syn::Ident,
}
