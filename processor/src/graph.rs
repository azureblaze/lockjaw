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
use crate::error::{compile_error, CompileError};
use crate::manifest::{Component, ComponentModuleManifest, Manifest, Type};
use crate::nodes::injectable::InjectableNode;
use crate::nodes::node::Node;
use crate::nodes::provides::ProvidesNode;
use crate::nodes::provision::ProvisionNode;
use proc_macro2::{Ident, TokenStream};
use quote::format_ident;
use quote::quote;
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::ops::Deref;

/// Dependency graph and other related data
#[derive(Default, Debug)]
pub struct Graph {
    map: HashMap<Ident, Box<dyn Node>>,
    module_manifest: ComponentModuleManifest,
    provisions: Vec<Box<ProvisionNode>>,
}

pub struct ComponentSections {
    fields: TokenStream,
    ctor_params: TokenStream,
    methods: TokenStream,
    trait_methods: TokenStream,
}

impl Debug for ComponentSections {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("")
            .field(&format!("fields: {}", self.fields.to_string()))
            .field(&format!("ctor_params: {}", self.ctor_params.to_string()))
            .field(&format!("methods: {}", self.methods.to_string()))
            .field(&format!(
                "trait_methods: {}",
                self.trait_methods.to_string()
            ))
            .finish()
    }
}

impl ComponentSections {
    pub fn new() -> Self {
        ComponentSections {
            fields: quote! {},
            ctor_params: quote! {},
            methods: quote! {},
            trait_methods: quote! {},
        }
    }

    pub fn merge(&mut self, other: ComponentSections) {
        let fields = &self.fields;
        let ctor_params = &self.ctor_params;
        let methods = &self.methods;
        let trait_methods = &self.trait_methods;

        let other_fields = &other.fields;
        let other_ctor_params = &other.ctor_params;
        let other_methods = &other.methods;
        let other_trait_methods = &other.trait_methods;

        self.fields = quote! {#fields #other_fields};
        self.ctor_params = quote! {#ctor_params #other_ctor_params};
        self.methods = quote! {#methods #other_methods};
        self.trait_methods = quote! {#trait_methods #other_trait_methods};
    }

    pub fn add_fields(&mut self, new_fields: TokenStream) {
        let fields = &self.fields;
        self.fields = quote! {#fields #new_fields}
    }

    pub fn add_ctor_params(&mut self, new_ctor_params: TokenStream) {
        let ctor_params = &self.ctor_params;
        self.ctor_params = quote! {#ctor_params #new_ctor_params}
    }

    pub fn add_methods(&mut self, new_methods: TokenStream) {
        let methods = &self.methods;
        self.methods = quote! {#methods #new_methods}
    }

    pub fn add_trait_methods(&mut self, new_trait_methods: TokenStream) {
        let trait_methods = &self.trait_methods;
        self.trait_methods = quote! {#trait_methods #new_trait_methods}
    }
}

pub fn generate_component(
    component: &Component,
    manifest: &Manifest,
) -> Result<(TokenStream, String), TokenStream> {
    let graph = crate::graph::build_graph(manifest, component)?;
    let component_name = component.field_type.syn_type();
    let component_impl_name = format_ident!(
        "{}Impl",
        component
            .field_type
            .local_string_path()
            .replace(" ", "")
            .replace("::", "_")
    );

    let mut component_sections = ComponentSections::new();

    component_sections.merge(graph.generate_modules());
    component_sections.merge(graph.generate_providers(component)?);

    let fields = &component_sections.fields;
    let ctor_params = &component_sections.ctor_params;
    let methods = &component_sections.methods;
    let trait_methods = &component_sections.trait_methods;
    let component_impl = quote! {
        #[allow(non_snake_case)]
        #[allow(non_camel_case_types)]
        struct #component_impl_name {
            #fields
        }
        #[allow(non_snake_case)]
        impl #component_impl_name {
            #methods
        }
        #[allow(non_snake_case)]
        impl #component_name for #component_impl_name {
            #trait_methods
        }
    };
    let mut builder = quote! {};
    if graph.module_manifest.field_type.is_some() {
        let module_manifest_name = graph.module_manifest.field_type.unwrap().syn_type();
        builder = quote! {
            impl dyn #component_name {
                #[allow(unused)]
                pub fn build (param : #module_manifest_name) -> Box<dyn #component_name>{
                   Box::new(#component_impl_name{#ctor_params})
                }
            }
        };
    }
    if graph.module_manifest.builder_modules.is_empty() {
        builder = quote! {
            #builder
            impl dyn #component_name {
                pub fn new () -> Box<dyn #component_name>{
                   Box::new(#component_impl_name{#ctor_params})
                }
            }
        };
    }

    Ok((
        quote! {
            #component_impl
            #builder
        },
        format!("graph: {:#?}", graph.map),
    ))
}

impl Graph {
    fn add_node(&mut self, node: &Box<dyn Node>) -> Result<(), TokenStream> {
        if self.map.contains_key(&node.get_type().identifier()) {
            let merged_node = self
                .map
                .get(&node.get_type().identifier())
                .expect("cannot find node")
                .merge(node.borrow())?;
            self.map
                .insert(merged_node.get_type().identifier(), merged_node);
        }
        self.map
            .insert(node.get_type().identifier(), node.clone_box());
        Ok(())
    }

    fn generate_modules(&self) -> ComponentSections {
        let mut result = ComponentSections::new();

        for module in &self.module_manifest.modules {
            let name = module.identifier();
            let path = module.syn_type();
            result.add_fields(quote! {
                #name : #path,
            });
            result.add_ctor_params(quote! {
                #name : #path {},
            });
        }

        for module in &self.module_manifest.builder_modules {
            let name = format_ident!("{}", module.name);
            let path = module.field_type.syn_type();
            result.add_fields(quote! {
                #name : #path,
            });
            result.add_ctor_params(quote! {
                #name : param.#name,
            });
        }

        result
    }

    fn generate_providers(&self, component: &Component) -> Result<ComponentSections, TokenStream> {
        let mut result = ComponentSections::new();
        let mut generated_nodes = HashSet::<Ident>::new();
        for provision in &self.provisions {
            result.merge(self.generate_provider(
                provision.deref(),
                component,
                &Vec::new(),
                &mut generated_nodes,
            )?);
        }
        Ok(result)
    }

    fn get_node(
        &self,
        type_: &Type,
        ancestors: &Vec<String>,
    ) -> Result<&Box<dyn Node>, TokenStream> {
        self.map
            .get(&type_.identifier())
            .map_compile_error(&format!(
                "missing bindings for {}\nrequested by: {} ",
                type_.readable(),
                ancestors.join("\nrequested by: ")
            ))
    }

    fn generate_provider(
        &self,
        node: &dyn Node,
        component: &Component,
        ancestors: &Vec<String>,
        generated_nodes: &mut HashSet<Ident>,
    ) -> Result<ComponentSections, TokenStream> {
        let mut result = ComponentSections::new();

        if ancestors.contains(&node.get_name()) {
            let l = ancestors
                .iter()
                .position(|s| s.eq(&node.get_name()))
                .unwrap();
            let mut s = String::new();
            for i in 0..ancestors.len() {
                if i == 0 {
                    s.push_str(&format!("*-- {}\n", ancestors.get(i).unwrap()));
                } else if i < l {
                    s.push_str(&format!("|   {}\n", ancestors.get(i).unwrap()));
                } else if i == l {
                    s.push_str(&format!("*-> {}\n", ancestors.get(i).unwrap()));
                } else {
                    s.push_str(&format!("    {}\n", ancestors.get(i).unwrap()));
                }
            }
            return compile_error(&format!("Cyclic dependency detected:\n{}", s));
        }

        if generated_nodes.contains(&node.get_identifier()) {
            return Ok(result);
        }

        generated_nodes.insert(node.get_identifier());
        result.merge(node.generate_provider(self)?);

        let mut new_ancestors = Vec::<String>::new();
        new_ancestors.push(node.get_name());
        new_ancestors.extend(ancestors.clone());
        for dependency in node.get_dependencies() {
            let dependency_node = self.get_node(dependency, ancestors)?;
            node.can_depend(dependency_node.borrow(), ancestors)?;
            result.merge(self.generate_provider(
                dependency_node.borrow(),
                component,
                &new_ancestors,
                generated_nodes,
            )?);
        }
        Ok(result)
    }

    pub fn has_scoped_deps(&self, node: &dyn Node) -> Result<bool, TokenStream> {
        for dep in node.get_dependencies() {
            let dep_node = self.get_node(dep, &Vec::new())?;
            if dep_node.is_scoped() {
                return Ok(true);
            }
            if self.has_scoped_deps(dep_node.borrow())? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

fn get_module_manifest(
    manifest: &Manifest,
    component: &Component,
) -> Result<ComponentModuleManifest, TokenStream> {
    if component.module_manifest.is_none() {
        return Ok(ComponentModuleManifest::new());
    }
    for module_manifest in &manifest.component_module_manifests {
        if module_manifest
            .field_type
            .as_ref()
            .unwrap()
            .identifier()
            .eq(&component.module_manifest.as_ref().unwrap().identifier())
        {
            return Ok(module_manifest.clone());
        }
    }
    compile_error(&format!(
        "cannot find module manifest {}, used by {}",
        component
            .module_manifest
            .as_ref()
            .unwrap()
            .canonical_string_path(),
        component.field_type.canonical_string_path()
    ))
}

fn build_graph(manifest: &Manifest, component: &Component) -> Result<Graph, TokenStream> {
    let mut result = Graph::default();
    for injectable in &manifest.injectables {
        let _: Vec<()> = InjectableNode::new(injectable)
            .iter()
            .map(|node| result.add_node(node))
            .collect::<Result<Vec<()>, TokenStream>>()?;
    }
    let mut installed_modules = HashSet::<Ident>::new();
    result.module_manifest = get_module_manifest(manifest, component)?;

    for module in &result.module_manifest.modules {
        installed_modules.insert(module.identifier());
    }

    for module in &result.module_manifest.builder_modules {
        installed_modules.insert(module.field_type.identifier());
    }

    for module in &manifest.modules {
        if !installed_modules.contains(&module.field_type.identifier()) {
            continue;
        }
        for provider in &module.providers {
            let _ = ProvidesNode::new(&result.module_manifest, &module.field_type, provider)
                .iter()
                .map(|node| result.add_node(node))
                .collect::<Result<Vec<()>, TokenStream>>()?;
        }
    }
    for provision in &component.provisions {
        result.provisions.push(Box::new(ProvisionNode::new(
            provision.clone(),
            component.clone(),
        )));
    }

    Ok(result)
}
