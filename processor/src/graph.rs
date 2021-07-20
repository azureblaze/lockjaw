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
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::ops::Deref;

use proc_macro2::{Ident, TokenStream};
use quote::format_ident;
use quote::quote;

use crate::error::compile_error;
use crate::manifest::{BindingType, BuilderModules, Component, ComponentType, Manifest, TypeRoot};
use crate::nodes::binds::BindsNode;
use crate::nodes::binds_option_of::BindsOptionOfNode;
use crate::nodes::injectable::InjectableNode;
use crate::nodes::node::Node;
use crate::nodes::parent::ParentNode;
use crate::nodes::provides::ProvidesNode;
use crate::nodes::provision::ProvisionNode;
use crate::nodes::subcomponent::SubcomponentNode;
use crate::type_data::TypeData;
use std::iter::FromIterator;

/// Dependency graph and other related data
#[derive(Default, Debug)]
pub struct Graph {
    pub map: HashMap<Ident, Box<dyn Node>>,
    pub modules: Vec<TypeData>,
    pub builder_modules: BuilderModules,
    pub provisions: Vec<Box<ProvisionNode>>,
}

pub struct ComponentSections {
    pub fields: TokenStream,
    pub ctor_params: TokenStream,
    pub methods: TokenStream,
    pub trait_methods: TokenStream,
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
    let (graph, missing_deps) = build_graph(manifest, component)?;
    if !missing_deps.is_empty() {
        let mut error = quote! {};
        for dep in missing_deps {
            let msg = &format!(
                "missing bindings for {}\nrequested by: {} ",
                dep.type_data.readable(),
                dep.ancestors
                    .iter()
                    .rev()
                    .map(|s| s.clone())
                    .collect::<Vec<String>>()
                    .join("\nrequested by: ")
            );
            error = quote! {
                #error
                compile_error!(#msg);
            }
        }
        return Err(error);
    }
    let component_name = component.type_data.syn_type();
    let component_impl_name = format_ident!(
        "{}Impl",
        component
            .type_data
            .local_string_path()
            .replace(" ", "")
            .replace("::", "_")
    );

    let mut component_sections = ComponentSections::new();

    component_sections.merge(graph.generate_modules());
    component_sections.merge(graph.generate_provisions(component)?);

    let fields = &component_sections.fields;
    let ctor_params = &component_sections.ctor_params;
    let methods = &component_sections.methods;
    let trait_methods = &component_sections.trait_methods;
    let component_impl = quote! {
        #[allow(non_snake_case)]
        #[allow(non_camel_case_types)]
        #[allow(dead_code)]
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
    if graph.builder_modules.type_data.is_some() {
        let module_manifest_name = graph.builder_modules.type_data.unwrap().syn_type();
        builder = quote! {
            impl dyn #component_name {
                #[allow(unused)]
                pub fn build (param : #module_manifest_name) -> Box<dyn #component_name>{
                   Box::new(#component_impl_name{#ctor_params})
                }
            }
        };
    }
    if graph.builder_modules.builder_modules.is_empty() {
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
    pub fn has_node(&self, type_data: &TypeData) -> bool {
        self.map.contains_key(&type_data.identifier())
    }

    fn add_node(&mut self, node: Box<dyn Node>) -> Result<(), TokenStream> {
        if self.map.contains_key(&node.get_type().identifier()) {
            let merged_node = self
                .map
                .get(&node.get_type().identifier())
                .expect("cannot find node")
                .merge(node.borrow())?;
            self.map
                .insert(merged_node.get_type().identifier(), merged_node);
        } else {
            self.map.insert(node.get_type().identifier(), node);
        }
        Ok(())
    }

    fn add_nodes(&mut self, nodes: Vec<Box<dyn Node>>) -> Result<(), TokenStream> {
        for node in nodes {
            self.add_node(node)?
        }
        Ok(())
    }

    pub fn generate_modules(&self) -> ComponentSections {
        let mut result = ComponentSections::new();

        for module in &self.modules {
            let name = module.identifier();
            let path = module.syn_type();
            result.add_fields(quote! {
                #name : #path,
            });
            result.add_ctor_params(quote! {
                #name : #path {},
            });
        }

        for module in &self.builder_modules.builder_modules {
            let name = format_ident!("{}", module.name);
            let path = module.type_data.syn_type();
            result.add_fields(quote! {
                #name : #path,
            });
            result.add_ctor_params(quote! {
                #name : param.#name,
            });
        }

        result
    }

    pub fn generate_provisions(
        &self,
        component: &Component,
    ) -> Result<ComponentSections, TokenStream> {
        let mut result = ComponentSections::new();
        let mut generated_nodes = HashSet::<Ident>::new();
        for provision in &self.provisions {
            result.merge(self.generate_provision(
                provision.deref(),
                component,
                &Vec::new(),
                &mut generated_nodes,
            )?);
        }
        Ok(result)
    }

    fn generate_provision(
        &self,
        node: &dyn Node,
        component: &Component,
        ancestors: &Vec<String>,
        generated_nodes: &mut HashSet<Ident>,
    ) -> Result<ComponentSections, TokenStream> {
        let mut result = ComponentSections::new();

        if generated_nodes.contains(&node.get_identifier()) {
            return Ok(result);
        }

        generated_nodes.insert(node.get_identifier());
        result.merge(node.generate_implementation(self)?);

        let mut new_ancestors = Vec::<String>::new();
        new_ancestors.push(node.get_name());
        new_ancestors.extend(ancestors.clone());
        for dependency in node.get_dependencies() {
            let dependency_node = self
                .map
                .get(&dependency.identifier())
                .expect(&format!("missing node for {}", dependency.readable()));
            result.merge(self.generate_provision(
                dependency_node.borrow(),
                component,
                &new_ancestors,
                generated_nodes,
            )?);
        }
        for dependency in node.get_optional_dependencies() {
            if !self.has_node(&dependency) {
                continue;
            }
            let dependency_node = self
                .map
                .get(&dependency.identifier())
                .expect(&format!("missing node for {}", dependency.readable()));
            result.merge(self.generate_provision(
                dependency_node.borrow(),
                component,
                &new_ancestors,
                generated_nodes,
            )?);
        }
        Ok(result)
    }

    pub fn has_scoped_deps(&self, identifier: &Ident) -> Result<bool, TokenStream> {
        let node = self.map.get(identifier).unwrap();
        for dep in node.get_dependencies() {
            let dep_node = self
                .map
                .get(&dep.identifier())
                .expect(&format!("missing node for {}", dep.readable()));
            if !dep_node.get_type().scopes.is_empty() {
                return Ok(true);
            }
            if self.has_scoped_deps(&dep.identifier())? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

fn get_module_manifest(
    manifest: &Manifest,
    component: &Component,
) -> Result<BuilderModules, TokenStream> {
    if component.builder_modules.is_none() {
        return Ok(BuilderModules::new());
    }
    for module_manifest in &manifest.builder_modules {
        if module_manifest
            .type_data
            .as_ref()
            .unwrap()
            .identifier()
            .eq(&component.builder_modules.as_ref().unwrap().identifier())
        {
            return Ok(module_manifest.clone());
        }
    }
    log!("{:#?}", manifest);
    compile_error(&format!(
        "cannot find module manifest {}, used by {}",
        component
            .builder_modules
            .as_ref()
            .unwrap()
            .canonical_string_path(),
        component.type_data.canonical_string_path()
    ))
}

pub struct MissingDependency {
    pub type_data: TypeData,
    pub ancestors: Vec<String>,
}

pub fn build_graph(
    manifest: &Manifest,
    component: &Component,
) -> Result<(Graph, Vec<MissingDependency>), TokenStream> {
    let mut result = Graph::default();
    let singleton = singleton_type();
    for injectable in &manifest.injectables {
        if injectable.type_data.scopes.is_empty()
            || injectable.type_data.scopes.contains(&component.type_data)
            || injectable.type_data.scopes.contains(&singleton)
        {
            result.add_node(InjectableNode::new(injectable))?;
        }
    }
    let mut installed_modules = HashSet::<Ident>::new();
    result.builder_modules = get_module_manifest(manifest, component)?;
    result.modules = component.modules.clone();

    let available_modules: HashSet<Ident> = manifest
        .modules
        .iter()
        .map(|m| m.type_data.identifier())
        .collect();
    for module in &result.modules {
        if !available_modules.contains(&module.identifier()) {
            return compile_error(&format!(
                "module {} not found, required by {}",
                &module.readable(),
                component.type_data.readable()
            ));
        }
    }

    for module in &result.builder_modules.builder_modules {
        if !available_modules.contains(&module.type_data.identifier()) {
            return compile_error(&format!(
                "module {} not found, required by {}",
                &module.type_data.readable(),
                component.type_data.readable()
            ));
        }
    }

    for module in &result.modules {
        installed_modules.insert(module.identifier());
    }

    for module in &result.builder_modules.builder_modules {
        installed_modules.insert(module.type_data.identifier());
    }
    for module in &manifest.modules {
        if !installed_modules.contains(&module.type_data.identifier()) {
            continue;
        }
        for binding in &module.bindings {
            if binding.type_data.scopes.is_empty()
                || binding.type_data.scopes.contains(&component.type_data)
                || binding.type_data.scopes.contains(&singleton)
            {
                result.add_nodes(match &binding.binding_type {
                    BindingType::Provides => {
                        ProvidesNode::new(&result.builder_modules, &module.type_data, binding)?
                    }
                    BindingType::Binds => {
                        BindsNode::new(&result.builder_modules, &module.type_data, binding)?
                    }
                    BindingType::BindsOptionOf => BindsOptionOfNode::new(binding),
                })?;
            }
        }
        for subcomponent in &module.subcomponents {
            result.add_node(SubcomponentNode::new(
                manifest,
                subcomponent,
                &component.type_data,
            )?)?;
        }
    }
    let mut resolved_nodes = HashSet::<Ident>::new();
    let mut missing_deps = Vec::new();
    for provision in &component.provisions {
        let provision = Box::new(ProvisionNode::new(provision.clone(), component.clone()));
        missing_deps.extend(resolve_dependencies(
            provision.as_ref(),
            &mut result.map,
            &mut vec![],
            &mut resolved_nodes,
        )?);
        result.provisions.push(provision);
    }
    if component.component_type == ComponentType::Subcomponent {
        for missing_dep in &missing_deps {
            result.add_node(ParentNode::new(&missing_dep.type_data)?)?;
        }
    }
    validate_graph(manifest, &result)?;
    Ok((result, missing_deps))
}

fn singleton_type() -> TypeData {
    let mut result = TypeData::new();
    result.root = TypeRoot::GLOBAL;
    result.path = "lockjaw::Singleton".to_string();
    result.field_crate = "lockjaw".to_string();
    result
}

fn resolve_dependencies(
    node: &dyn Node,
    map: &mut HashMap<Ident, Box<dyn Node>>,
    ancestors: &mut Vec<String>,
    resolved_nodes: &mut HashSet<Ident>,
) -> Result<Vec<MissingDependency>, TokenStream> {
    if ancestors.contains(&node.get_name()) {
        return cyclic_dependency(node, ancestors);
    }

    if resolved_nodes.contains(&node.get_identifier()) {
        return Ok(Vec::new());
    }

    resolved_nodes.insert(node.get_identifier());
    let mut missing_deps = Vec::<MissingDependency>::new();

    ancestors.push(node.get_name());
    for dependency in node.get_dependencies() {
        let mut dependency_node = map.get(&dependency.identifier());

        if dependency_node.is_none() {
            if let Some(generated_node) = <dyn Node>::generate_node(&dependency) {
                let identifier = generated_node.get_identifier();
                map.insert(identifier.clone(), generated_node);
                dependency_node = map.get(&identifier);
            } else {
                missing_deps.push(MissingDependency {
                    type_data: dependency.clone(),
                    ancestors: ancestors.clone(),
                });
                continue;
            }
        }
        let cloned_node = dependency_node.unwrap().clone_box();
        node.can_depend(cloned_node.as_ref(), ancestors)?;
        missing_deps.extend(resolve_dependencies(
            cloned_node.as_ref(),
            map,
            ancestors,
            resolved_nodes,
        )?);
    }
    for dependency in node.get_optional_dependencies() {
        let mut dependency_node = map.get(&dependency.identifier());
        if dependency_node.is_none() {
            let generated_node = <dyn Node>::generate_node(&dependency);
            if generated_node.is_none() {
                continue;
            }
            let identifier = generated_node.as_ref().unwrap().get_identifier();
            map.insert(identifier.clone(), generated_node.unwrap());
            dependency_node = map.get(&identifier);
        }
        let cloned_node = dependency_node.unwrap().clone_box();
        node.can_depend(cloned_node.as_ref(), ancestors)?;
        missing_deps.extend(resolve_dependencies(
            cloned_node.as_ref(),
            map,
            ancestors,
            resolved_nodes,
        )?);
    }
    ancestors.pop();
    Ok(missing_deps)
}

fn cyclic_dependency<T>(node: &dyn Node, ancestors: &mut Vec<String>) -> Result<T, TokenStream> {
    ancestors.push(node.get_name());
    ancestors.reverse();
    let mut iter = ancestors.iter();
    iter.next();
    let chain_start = iter.position(|s| s.eq(&node.get_name())).unwrap() + 1;
    let mut s: Vec<String> = vec![];
    for i in 0..ancestors.len() {
        if i == 0 {
            s.push(format!("*-- {}", ancestors.get(i).unwrap()));
        } else if i < chain_start {
            s.push(format!("|   {}", ancestors.get(i).unwrap()));
        } else if i == chain_start {
            s.push(format!("*-> {}", ancestors.get(i).unwrap()));
        } else {
            s.push(format!("    {}", ancestors.get(i).unwrap()));
        }
    }
    return compile_error(&format!("Cyclic dependency detected:\n{}", s.join("\n")));
}

fn validate_graph(manifest: &Manifest, graph: &Graph) -> Result<(), TokenStream> {
    let qualifiers: HashSet<TypeData> = HashSet::from_iter(manifest.qualifiers.clone());
    for node in graph.map.values() {
        if let Some(ref qualifier) = node.get_type().qualifier {
            if !qualifiers.contains(qualifier) {
                return compile_error(&format!(
                    "{} binds {} with a qualifier, but the qualifier struct is not annotated with \
                    the #[lockjaw::qualifier] attribute",
                    node.get_name(),
                    node.get_type().readable()
                ));
            }
        }
    }
    Ok(())
}
