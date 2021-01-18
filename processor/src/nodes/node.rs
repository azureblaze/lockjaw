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
use crate::error::compile_error;
use crate::graph::{ComponentSections, Graph};
use crate::nodes::maybe_scoped::MaybeScopedNode;
use crate::nodes::scoped::ScopedNode;
use crate::protos::manifest::{ComponentModuleManifest, Type, Type_Root};
use proc_macro2::{Ident, TokenStream};
use quote::format_ident;
use std::fmt::Debug;

pub trait Node: Debug {
    fn get_name(&self) -> String;
    fn generate_provider(&self, graph: &Graph) -> Result<ComponentSections, TokenStream>;
    fn merge(&self, new_node: &dyn Node) -> Result<Box<dyn Node>, TokenStream> {
        Node::duplicated_impl(
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
            return Node::no_scope(target_node, ancestors);
        }
        Ok(())
    }

    fn get_type(&self) -> &Type;
    fn get_identifier(&self) -> Ident {
        self.get_type().identifier()
    }
    fn get_dependencies(&self) -> &Vec<Type>;
    fn is_scoped(&self) -> bool;

    fn set_scoped(&mut self, scoped: bool);

    fn clone_box(&self) -> Box<dyn Node>;
}

impl dyn Node {
    pub fn duplicated<T>(node: &dyn Node, new_node: &dyn Node) -> Result<T, TokenStream> {
        Node::duplicated_impl(
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
        if !node.get_type().get_scopes().is_empty() {
            let mut private_node = node.clone_box();
            private_node.set_scoped(true);

            let scoped_node = Box::new(ScopedNode {
                type_: Node::ref_type(&node.get_type()),
                dependencies: vec![private_node.get_type().clone()],
                scoped: false,
                node: private_node.clone_box(),
            });

            let maybe_scoped_node = Box::new(MaybeScopedNode {
                type_: Node::maybe_scoped_type(&private_node.get_type()),
                dependencies: vec![private_node.get_type().clone()],
                scoped: false,

                node: private_node.clone_box(),
            });

            return vec![private_node, scoped_node, maybe_scoped_node];
        }

        if node.get_type().scopes.is_empty() {
            if node.get_type().get_path().ne("lockjaw::MaybeScoped") {
                let boxed_node = Box::new(MaybeScopedNode {
                    type_: Node::maybe_scoped_type(&node.get_type()),
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
        module_type: &Type,
    ) -> ModuleInstance {
        let ident = module_type.identifier();
        for module in manifest.get_modules() {
            if module.identifier().eq(&ident) {
                return ModuleInstance {
                    type_: module_type.clone(),
                    name: module.identifier(),
                };
            }
        }

        for module in manifest.get_builder_modules() {
            if module.get_field_type().identifier().eq(&ident) {
                return ModuleInstance {
                    type_: module_type.clone(),
                    name: format_ident!("{}", module.get_name().to_owned()),
                };
            }
        }

        panic!("requested module not in manifest")
    }

    pub fn maybe_scoped_type(type_: &Type) -> Type {
        let mut boxed_type = Type::new();
        boxed_type.set_root(Type_Root::GLOBAL);
        boxed_type.set_path("lockjaw::MaybeScoped".to_string());
        boxed_type.mut_args().push(type_.clone());
        boxed_type
    }

    pub fn ref_type(type_: &Type) -> Type {
        let mut ref_type = type_.clone();
        ref_type.set_field_ref(true);
        ref_type
    }
}

/// An item in a module
#[derive(Debug, Clone)]
pub struct ModuleInstance {
    pub type_: Type,
    pub name: syn::Ident,
}
