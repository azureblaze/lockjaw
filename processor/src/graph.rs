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
use crate::manifest::ProcessorComponent;
use crate::nodes::binds::BindsNode;
use crate::nodes::binds_option_of::BindsOptionOfNode;
use crate::nodes::entry_point::EntryPointNode;
use crate::nodes::injectable::InjectableNode;
use crate::nodes::map::MapNode;
use crate::nodes::node::Node;
use crate::nodes::parent::ParentNode;
use crate::nodes::provides::ProvidesNode;
use crate::nodes::provision::ProvisionNode;
use crate::nodes::scoped::ScopedNode;
use crate::nodes::subcomponent::SubcomponentNode;
use crate::nodes::vec::VecNode;
use crate::type_data::ProcessorTypeData;
use crate::{component_visibles, components};
use lockjaw_common::manifest::{
    BindingType, BuilderModules, Component, ComponentType, Manifest, MultibindingType, TypeRoot,
};
use lockjaw_common::type_data::TypeData;
use std::iter::FromIterator;

/// Dependency graph and other related data
#[derive(Debug)]
pub struct Graph<'a> {
    pub component: Component,
    pub map: HashMap<Ident, Box<dyn Node>>,
    pub modules: HashSet<TypeData>,
    pub builder_modules: BuilderModules,
    pub root_nodes: Vec<Box<dyn Node>>,
    pub manifest: &'a Manifest,
}

pub struct ComponentSections {
    pub fields: TokenStream,
    pub ctor_params: TokenStream,
    pub ctor_statements: TokenStream,
    pub methods: TokenStream,
    pub trait_methods: TokenStream,
    pub items: TokenStream,
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
            .field(&format!("items: {}", self.items.to_string()))
            .finish()
    }
}

impl ComponentSections {
    pub fn new() -> Self {
        ComponentSections {
            fields: quote! {},
            ctor_params: quote! {},
            ctor_statements: quote! {},
            methods: quote! {},
            trait_methods: quote! {},
            items: quote! {},
        }
    }

    pub fn merge(&mut self, other: ComponentSections) {
        let fields = &self.fields;
        let ctor_params = &self.ctor_params;
        let ctor_statements = &self.ctor_statements;
        let methods = &self.methods;
        let trait_methods = &self.trait_methods;
        let items = &self.items;

        let other_fields = &other.fields;
        let other_ctor_params = &other.ctor_params;
        let other_ctor_statements = &other.ctor_statements;
        let other_methods = &other.methods;
        let other_trait_methods = &other.trait_methods;
        let other_items = &other.items;

        self.fields = quote! {#fields #other_fields};
        self.ctor_params = quote! {#ctor_params #other_ctor_params};
        self.ctor_statements = quote! {#ctor_statements #other_ctor_statements};
        self.methods = quote! {#methods #other_methods};
        self.trait_methods = quote! {#trait_methods #other_trait_methods};
        self.items = quote! {#items #other_items};
    }

    pub fn add_fields(&mut self, new_fields: TokenStream) {
        let fields = &self.fields;
        self.fields = quote! {#fields #new_fields}
    }

    pub fn add_ctor_params(&mut self, new_ctor_params: TokenStream) {
        let ctor_params = &self.ctor_params;
        self.ctor_params = quote! {#ctor_params #new_ctor_params}
    }

    pub fn add_ctor_statements(&mut self, new_ctor_statements: TokenStream) {
        let ctor_statements = &self.ctor_statements;
        self.ctor_statements = quote! {#ctor_statements #new_ctor_statements}
    }

    pub fn add_methods(&mut self, new_methods: TokenStream) {
        let methods = &self.methods;
        self.methods = quote! {#methods #new_methods}
    }

    pub fn add_trait_methods(&mut self, new_trait_methods: TokenStream) {
        let trait_methods = &self.trait_methods;
        self.trait_methods = quote! {#trait_methods #new_trait_methods}
    }

    pub fn add_items(&mut self, new_items: TokenStream) {
        let items = &self.items;
        self.items = quote! {#items #new_items}
    }
}

pub fn generate_component(
    component: &Component,
    manifest: &Manifest,
) -> Result<(TokenStream, String), TokenStream> {
    let (graph, missing_deps) = build_graph(manifest, component, &Vec::new())?;
    if !missing_deps.is_empty() {
        let mut error = quote! {};
        for dep in missing_deps {
            let msg = format!(
                "missing bindings for {}\n{}",
                dep.type_data.readable(),
                dep.to_message()
            );
            error = quote! {
                #error
                compile_error!(#msg);
            }
        }
        return Err(error);
    }
    let component_name = component.type_data.syn_type();
    let component_impl_name = component.impl_ident();

    let mut component_sections = ComponentSections::new();

    component_sections.merge(graph.generate_modules(&manifest));
    component_sections.merge(graph.generate_provisions(component)?);

    let fields = &component_sections.fields;
    let ctor_params = &component_sections.ctor_params;
    let ctor_statements = &component_sections.ctor_statements;
    let methods = &component_sections.methods;
    let trait_methods = &component_sections.trait_methods;
    let items = &component_sections.items;

    let component_impl = quote! {
        #[doc(hidden)]
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
        #items
    };

    let builder_name = components::builder_name(&component.type_data);

    let builder = if graph.builder_modules.type_data.is_some() {
        let module_manifest_name = graph.builder_modules.type_data.unwrap().syn_type();
        quote! {
            #[doc(hidden)]
            #[no_mangle]
            #[allow(non_snake_case)]
            fn #builder_name (param : #module_manifest_name) -> Box<dyn #component_name>{
                #ctor_statements
                Box::new(#component_impl_name{#ctor_params})
            }
        }
    } else {
        quote! {
            #[doc(hidden)]
            #[no_mangle]
            #[allow(non_snake_case)]
            fn #builder_name () -> Box<dyn #component_name>{
                #ctor_statements
                Box::new(#component_impl_name{#ctor_params})
            }
        }
    };

    Ok((
        quote! {
            #component_impl
            #builder
        },
        format!("graph: {:#?}", graph.map),
    ))
}

impl<'a> Graph<'a> {
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

    pub fn generate_modules(&self, manifest: &Manifest) -> ComponentSections {
        let mut result = ComponentSections::new();

        for module in &self.modules {
            let name = module.identifier();
            let path = component_visibles::visible_type(manifest, &module).syn_type();
            result.add_fields(quote! {
                #name : #path,
            });
            result.add_ctor_params(quote! {
                #name : #path {},
            });
        }

        for module in &self.builder_modules.builder_modules {
            let name = format_ident!("{}", module.name);
            let path = component_visibles::visible_type(manifest, &module.type_data).syn_type();
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
        for provision in &self.root_nodes {
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
                .get(&dependency.type_.identifier())
                .expect(&format!(
                    "missing node for {}, depended by {}",
                    dependency.type_.identifier().to_string(),
                    node.get_name()
                ));
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

    pub fn has_lifetime(&self, type_: &TypeData) -> bool {
        if type_.path == "lockjaw::Cl" {
            return true;
        }
        return self.manifest.lifetimed_types.contains(type_);
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
    pub multibinding_type: MultibindingType,
    pub message: String,
}

impl MissingDependency {
    pub fn to_message(&self) -> String {
        if self.message.is_empty() {
            format!(
                "requested by: {} ",
                self.ancestors
                    .iter()
                    .rev()
                    .map(|s| s.clone())
                    .collect::<Vec<String>>()
                    .join("\nrequested by: ")
            )
        } else {
            format!(
                "from: \n\t{}\n\
                requested by: {} ",
                self.message.replace("\n", "\n\t"),
                self.ancestors
                    .iter()
                    .rev()
                    .map(|s| s.clone())
                    .collect::<Vec<String>>()
                    .join("\nrequested by: ")
            )
        }
    }
}

pub fn build_graph<'a>(
    manifest: &'a Manifest,
    component: &Component,
    parent_multibinding_nodes: &Vec<Box<dyn Node>>,
) -> Result<(Graph<'a>, Vec<MissingDependency>), TokenStream> {
    let mut result = Graph {
        component: Default::default(),
        map: Default::default(),
        modules: Default::default(),
        builder_modules: Default::default(),
        root_nodes: vec![],
        manifest,
    };
    result.component = component.clone();
    let singleton = singleton_type();
    for node in parent_multibinding_nodes {
        result.add_node(node.clone_box())?;
    }

    for injectable in &manifest.injectables {
        if injectable.type_data.scopes.is_empty()
            || injectable.type_data.scopes.contains(&component.type_data)
            || injectable.type_data.scopes.contains(&singleton)
        {
            result.add_node(InjectableNode::new(injectable))?;
            if !injectable.type_data.scopes.is_empty() {
                let mut ref_type = injectable.type_data.clone();
                ref_type.field_ref = true;
                ref_type.scopes = HashSet::new();
                result.add_node(ScopedNode::for_type(&ref_type))?;
            }
        }
    }
    let mut installed_modules = HashSet::<Ident>::new();
    result.builder_modules = get_module_manifest(manifest, component)?;
    result.modules = HashSet::from_iter(component.modules.clone());

    for module in &manifest.modules {
        if module.install_in.contains(&component.type_data)
            || (component.component_type == ComponentType::Component
                && module.install_in.contains(&singleton_type()))
        {
            if !component.definition_only {
                if module.install_in.contains(&singleton_type()) {
                    continue;
                }
                if module.bindings.is_empty() && module.subcomponents.len() == 1 {
                    return compile_error(
                        &format!("#[subcomponent] {} has `parent` {},\
                     but the component is not annotated with #[define_component] or #[define_subcomponent]",
                                 module.subcomponents.iter().next().unwrap().readable(),
                                 component.type_data.readable()));
                }
                return compile_error(
                    &format!("#[module] {} is `install_in` {},\
                     but the component is not annotated with #[define_component] or #[define_subcomponent]",
                             module.type_data.readable(),
                             component.type_data.readable()));
            }
            result.modules.insert(module.type_data.clone());
        }
    }

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
                    BindingType::Multibinds => match binding.type_data.path.as_str() {
                        "std::vec::Vec" => {
                            let mut type_ = binding.type_data.args[0].clone();
                            type_.qualifier = binding.type_data.qualifier.clone();
                            vec![VecNode::new(&type_)]
                        }
                        "std::collections::HashMap" => {
                            let mut type_ = binding.type_data.args[1].clone();
                            type_.qualifier = binding.type_data.qualifier.clone();
                            vec![MapNode::with_key_type(&binding.type_data.args[0], &type_)?]
                        }
                        _ => {
                            panic!("unexpected type for multibinds");
                        }
                    },
                })?;
            }
        }
    }
    let mut multibinding_nodes: Vec<Box<dyn Node>> = Vec::new();

    for (_, v) in result.map.iter() {
        if let Some(vec_node) = v.as_any().downcast_ref::<VecNode>() {
            let mut sub_vec_node = VecNode::new(&vec_node.type_.args[0]);
            for binding in &vec_node.bindings {
                let parent_node = ParentNode::new(&MissingDependency {
                    type_data: binding.type_data.clone(),
                    ancestors: Vec::new(),
                    message: String::new(),
                    multibinding_type: binding.multibinding_type.clone(),
                })?;
                sub_vec_node.add_binding(&binding.type_data, &binding.multibinding_type);
                multibinding_nodes.push(parent_node);
            }
            multibinding_nodes.push(sub_vec_node);
        } else if let Some(map_node) = v.as_any().downcast_ref::<MapNode>() {
            let mut sub_map_node =
                MapNode::with_key_type(&map_node.type_.args[0], &map_node.type_.args[1])?;
            for (key, binding) in &map_node.bindings {
                let parent_node = ParentNode::new(&MissingDependency {
                    type_data: binding.clone(),
                    message: String::new(),
                    ancestors: Vec::new(),
                    multibinding_type: MultibindingType::IntoMap,
                })?;
                sub_map_node.add_binding(key, parent_node.get_type());
                multibinding_nodes.push(parent_node);
            }
            multibinding_nodes.push(sub_map_node);
        }
    }
    let mut subcomponents = HashSet::<TypeData>::new();
    for module in &manifest.modules {
        if !installed_modules.contains(&module.type_data.identifier()) {
            continue;
        }
        for subcomponent in &module.subcomponents {
            subcomponents.insert(subcomponent.clone());
        }
    }
    for subcomponent in &subcomponents {
        result.add_nodes(SubcomponentNode::new(
            manifest,
            subcomponent,
            &component.type_data,
            &multibinding_nodes,
        )?)?;
    }

    let mut resolved_nodes = HashSet::<Ident>::new();
    let mut missing_deps = Vec::new();
    for provision in &component.provisions {
        let provision = Box::new(ProvisionNode::new(provision.clone(), component.clone()));
        missing_deps.extend(resolve_dependencies(
            provision.as_ref(),
            &mut result.map,
            vec![],
            vec![],
            &mut resolved_nodes,
        )?);
        result.root_nodes.push(provision);
    }

    for entry_point in &manifest.entry_points {
        if entry_point.component.canonical_string_path()
            == component.type_data.canonical_string_path()
        {
            if !component.definition_only {
                return compile_error(
                    &format!("#[entry_point] {} is `install_in` {},\
                     but the component is not annotated with #[define_component] or #[define_subcomponent]",
                             entry_point.type_data.readable(),
                             component.type_data.readable()));
            }
            let node = Box::new(EntryPointNode::new(entry_point));
            missing_deps.extend(resolve_dependencies(
                node.as_ref(),
                &mut result.map,
                vec![],
                vec![],
                &mut resolved_nodes,
            )?);
            result.root_nodes.push(node);
        }
    }

    if component.component_type == ComponentType::Subcomponent {
        for (_, v) in &mut result.map {
            if let Some(vec_node) = v.as_mut_any().downcast_mut::<VecNode>() {
                missing_deps.push(MissingDependency {
                    type_data: vec_node.type_.clone(),
                    message: String::new(),
                    ancestors: vec![format!("({} into_vec)", component.type_data.readable())],
                    multibinding_type: MultibindingType::IntoVec,
                });
            } else if let Some(map_node) = v.as_mut_any().downcast_mut::<MapNode>() {
                missing_deps.push(MissingDependency {
                    type_data: map_node.type_.clone(),
                    message: String::new(),
                    ancestors: vec![format!("({} into_map)", component.type_data.readable())],
                    multibinding_type: MultibindingType::IntoMap,
                });
            }
        }

        for missing_dep in &missing_deps {
            result.add_node(ParentNode::new(&missing_dep)?)?;
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
    mut ancestors: Vec<String>,
    mut static_ancestors: Vec<String>,
    resolved_nodes: &mut HashSet<Ident>,
) -> Result<Vec<MissingDependency>, TokenStream> {
    if static_ancestors.contains(&node.get_name()) {
        return cyclic_dependency(node, &mut ancestors);
    }

    if resolved_nodes.contains(&node.get_identifier()) {
        return Ok(Vec::new());
    }

    resolved_nodes.insert(node.get_identifier());
    let mut missing_deps = Vec::<MissingDependency>::new();

    if node.is_runtime_dependency() {
        static_ancestors.clear();
    } else {
        static_ancestors.push(node.get_name());
    }
    ancestors.push(node.get_name());
    for dependency in node.get_dependencies() {
        let mut dependency_node = map.get(&dependency.type_.identifier());

        if dependency_node.is_none() {
            if let Some(generated_node) = <dyn Node>::generate_node(map, &dependency.type_) {
                let identifier = generated_node.get_identifier();
                map.insert(identifier.clone(), generated_node);
                dependency_node = map.get(&identifier);
            } else {
                missing_deps.push(MissingDependency {
                    type_data: dependency.type_.clone(),
                    message: dependency.message,
                    ancestors: ancestors.clone(),
                    multibinding_type: MultibindingType::None,
                });
                continue;
            }
        }
        let cloned_node = dependency_node.unwrap().clone_box();
        node.can_depend(cloned_node.as_ref(), &ancestors)?;
        missing_deps.extend(resolve_dependencies(
            cloned_node.as_ref(),
            map,
            ancestors.clone(),
            static_ancestors.clone(),
            resolved_nodes,
        )?);
    }
    for dependency in node.get_optional_dependencies() {
        let mut dependency_node = map.get(&dependency.identifier());
        if dependency_node.is_none() {
            let generated_node = <dyn Node>::generate_node(map, &dependency);
            if generated_node.is_none() {
                continue;
            }
            let identifier = generated_node.as_ref().unwrap().get_identifier();
            map.insert(identifier.clone(), generated_node.unwrap());
            dependency_node = map.get(&identifier);
        }
        let cloned_node = dependency_node.unwrap().clone_box();
        node.can_depend(cloned_node.as_ref(), &ancestors)?;
        missing_deps.extend(resolve_dependencies(
            cloned_node.as_ref(),
            map,
            ancestors.clone(),
            static_ancestors.clone(),
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
