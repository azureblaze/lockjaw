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
#[allow(unused)]
use crate::log;
use crate::protos::manifest::{
    Component, ComponentModuleManifest, Injectable, Manifest, Provider, Type, Type_Root,
};
use proc_macro2::{Ident, TokenStream};
use quote::format_ident;
use quote::quote;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::ops::Deref;
use syn::export::Formatter;

/// Dependency graph and other related data
#[derive(Default, Debug)]
pub struct Graph {
    map: HashMap<Ident, Node>,
    module_manifest: ComponentModuleManifest,
}

/// A node can create `type_` with `dependencies`
#[derive(Debug, Clone)]
pub struct Node {
    type_: Type,
    dependencies: Vec<Type>,
    node_type: NodeType,
    scoped: bool,
}

/// An item in a module
#[derive(Debug, Clone)]
pub struct ModuleInstance {
    type_: Type,
    name: syn::Ident,
}

#[derive(Debug, Clone)]
pub enum NodeType {
    /// A node created by #[injectable]
    Injectable(Injectable),
    /// A node created by #[provides]
    Provides(ModuleInstance, Provider),
    /// A node created by #[binds]
    Binds(ModuleInstance, Provider),
    /// Auto created node to provide Box<T> from T
    Boxed(Box<Node>),
    /// A node created by #[injectable/provides/binds (scope="..")]
    Scoped(Box<Node>),
}

trait NodeVisitor<R> {
    fn accept_injectable(&self, node: &Node, injectable: &Injectable) -> R;
    fn accept_provides(&self, node: &Node, module: &ModuleInstance, provides: &Provider) -> R;
    fn accept_binds(&self, node: &Node, module: &ModuleInstance, provides: &Provider) -> R;
    fn accept_boxed(&self, node: &Node, boxed_node: &Node) -> R;
    fn accept_scoped(&self, node: &Node, boxed_node: &Node) -> R;
}

impl Node {
    fn visit<R, V: NodeVisitor<R>>(&self, visitor: V) -> R {
        match self.node_type {
            NodeType::Injectable(ref injectable) => visitor.accept_injectable(&self, injectable),
            NodeType::Provides(ref module, ref provides) => {
                visitor.accept_provides(&self, &module, provides)
            }
            NodeType::Binds(ref module, ref provides) => {
                visitor.accept_binds(&self, &module, provides)
            }
            NodeType::Boxed(ref node) => visitor.accept_boxed(&self, node.deref()),
            NodeType::Scoped(ref node) => visitor.accept_scoped(&self, node.deref()),
        }
    }

    fn get_name(&self) -> String {
        self.visit(ProviderNameVisitor {})
    }
}

struct ComponentSections {
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
    fn new() -> Self {
        ComponentSections {
            fields: quote! {},
            ctor_params: quote! {},
            methods: quote! {},
            trait_methods: quote! {},
        }
    }

    fn merge(&mut self, other: ComponentSections) {
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

    fn add_fields(&mut self, new_fields: TokenStream) {
        let fields = &self.fields;
        self.fields = quote! {#fields #new_fields}
    }

    fn add_ctor_params(&mut self, new_ctor_params: TokenStream) {
        let ctor_params = &self.ctor_params;
        self.ctor_params = quote! {#ctor_params #new_ctor_params}
    }

    fn add_methods(&mut self, new_methods: TokenStream) {
        let methods = &self.methods;
        self.methods = quote! {#methods #new_methods}
    }

    fn add_trait_methods(&mut self, new_trait_methods: TokenStream) {
        let trait_methods = &self.trait_methods;
        self.trait_methods = quote! {#trait_methods #new_trait_methods}
    }
}

pub fn generate_component(
    component: &Component,
    manifest: &Manifest,
) -> Result<TokenStream, TokenStream> {
    let graph = crate::graph::build_graph(manifest, component)?;
    let component_name = component.get_field_type().syn_type();
    let component_impl_name = format_ident!(
        "{}Impl",
        component
            .get_field_type()
            .local_string_path()
            .replace(" ", "")
            .replace("::", "_")
    );

    let mut component_sections = ComponentSections::new();

    component_sections.merge(graph.generate_modules());
    component_sections.merge(graph.generate_providers(component)?);
    component_sections.merge(graph.generate_provisions(component)?);

    let fields = &component_sections.fields;
    let ctor_params = &component_sections.ctor_params;
    let methods = &component_sections.methods;
    let trait_methods = &component_sections.trait_methods;

    let component_impl = quote! {
        struct #component_impl_name {
            #fields
        }

        impl #component_impl_name {
            #methods
        }

        impl #component_name for #component_impl_name {
            #trait_methods
        }
    };
    let mut builder = quote! {};
    if graph.module_manifest.has_field_type() {
        let module_manifest_name = graph.module_manifest.get_field_type().syn_type();
        builder = quote! {
            impl dyn #component_name {
                pub fn build (param : #module_manifest_name) -> Box<dyn #component_name>{
                   Box::new(#component_impl_name{#ctor_params})
                }
            }
        };
    }
    if graph.module_manifest.get_builder_modules().is_empty() {
        builder = quote! {
            #builder
            impl dyn #component_name {
                pub fn new () -> Box<dyn #component_name>{
                   Box::new(#component_impl_name{#ctor_params})
                }
            }
        };
    }

    Ok(quote! {
        #component_impl
        #builder
    })
}

impl Graph {
    fn add_node(&mut self, node: &Node) -> Result<(), TokenStream> {
        if self.map.contains_key(&node.type_.identifier()) {
            let merged_node = self
                .map
                .get(&node.type_.identifier())
                .expect("cannot find node")
                .visit(MergeNodeVisitor { new_node: &node })?;
            self.map.insert(merged_node.type_.identifier(), merged_node);
        }
        self.map.insert(node.type_.identifier(), node.clone());
        Ok(())
    }

    fn generate_modules(&self) -> ComponentSections {
        let mut result = ComponentSections::new();

        for module in self.module_manifest.get_modules() {
            let name = module.identifier();
            let path = module.syn_type();
            result.add_fields(quote! {
                #name : #path,
            });
            result.add_ctor_params(quote! {
                #name : #path {},
            });
        }

        for module in self.module_manifest.get_builder_modules() {
            let name = format_ident!("{}", module.get_name());
            let path = module.get_field_type().syn_type();
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
        for provision in component.get_provisions() {
            let mut ancestors = Vec::<String>::new();
            ancestors.push(format!(
                "{}.{}",
                component.get_field_type().canonical_string_path(),
                provision.get_name()
            ));
            let dependency_node = self.get_node(provision.get_field_type(), &ancestors)?;
            if dependency_node.scoped {
                return compile_error(&format! {
                    "unable to provide scoped binding as regular object {}\nrequested by:{}",
                    dependency_node.get_name(),
                    ancestors.join("\nrequested by:")
                });
            }
            result.merge(self.generate_provider(
                dependency_node,
                component,
                &ancestors,
                &mut generated_nodes,
            )?);
        }
        Ok(result)
    }

    fn generate_provisions(&self, component: &Component) -> Result<ComponentSections, TokenStream> {
        let mut result = ComponentSections::new();
        for dependency in component.get_provisions() {
            let dependency_name = format_ident!("{}", dependency.get_name());
            let dep_node = self.get_node(dependency.get_field_type(), &Vec::new())?;
            let bound;
            if self.has_scoped_deps(dep_node)? {
                bound = "_";
            } else {
                bound = "";
            }
            let dependency_path = dependency
                .get_field_type()
                .syn_type_with_innner_lifetime_bound(bound);
            let dependency_type;
            if dependency.get_field_type().get_field_ref() {
                dependency_type = quote! {& #dependency_path};
            } else {
                dependency_type = quote! {#dependency_path}
            }
            let provider_name = dependency.get_field_type().identifier();
            result.add_trait_methods(quote! {
               fn #dependency_name(&self) -> #dependency_type {
                  self.#provider_name()
               }
            });
        }
        Ok(result)
    }

    fn get_node(&self, type_: &Type, ancestors: &Vec<String>) -> Result<&Node, TokenStream> {
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
        node: &Node,
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

        if generated_nodes.contains(&node.type_.identifier()) {
            return Ok(result);
        }

        generated_nodes.insert(node.type_.identifier());
        result.merge(node.visit(ProviderGenerator { graph: self })?);

        let mut new_ancestors = Vec::<String>::new();
        new_ancestors.push(node.get_name());
        new_ancestors.extend(ancestors.clone());
        for dependency in &node.dependencies {
            let dependency_node = self.get_node(dependency, ancestors)?;
            node.visit(CanDepend {
                target_node: dependency_node,
                ancestors,
            })?;
            result.merge(self.generate_provider(
                dependency_node,
                component,
                &new_ancestors,
                generated_nodes,
            )?);
        }
        Ok(result)
    }

    fn has_scoped_deps(&self, node: &Node) -> Result<bool, TokenStream> {
        for dep in &node.dependencies {
            let dep_node = self.get_node(dep, &Vec::new())?;
            if dep_node.scoped {
                return Ok(true);
            }
            if self.has_scoped_deps(dep_node)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

/// Generates node code.
struct ProviderGenerator<'a> {
    graph: &'a Graph,
}
impl NodeVisitor<Result<ComponentSections, TokenStream>> for ProviderGenerator<'_> {
    fn accept_injectable(
        &self,
        node: &Node,
        injectable: &Injectable,
    ) -> Result<ComponentSections, TokenStream> {
        let has_ref = self.graph.has_scoped_deps(node)?;
        let mut params = quote! {};
        for field in injectable.get_fields() {
            if field.get_injected() {
                let param_name = format_ident!("{}", field.get_name());
                let param_type = field.get_field_type().syn_type();
                if field.get_field_type().get_field_ref() {
                    params = quote! {
                       #params #param_name : &'a #param_type,
                    };
                } else {
                    params = quote! {
                       #params #param_name : #param_type,
                    }
                }
            }
        }
        let mut ctor_params = quote! {};
        for field in &injectable.fields {
            let param_name = format_ident!("{}", field.get_name());
            if field.get_injected() {
                let param_provider_name = field.get_field_type().identifier();
                ctor_params = quote! {
                   #ctor_params
                   #param_name : self.#param_provider_name(),
                }
            } else {
                let param_type = field.get_field_type().syn_type();
                ctor_params = quote! {
                   #ctor_params
                   #param_name : <#param_type>::default(),
                }
            }
        }
        let lifetime;
        if has_ref {
            lifetime = quote! {<'_>};
        } else {
            lifetime = quote! {};
        }

        let name_ident = node.type_.identifier();
        let injectable_path = node.type_.syn_type();
        let mut result = ComponentSections::new();
        result.add_methods(quote! {
            fn #name_ident(&'_ self) -> #injectable_path #lifetime{
                #injectable_path{#ctor_params}
            }
        });
        Ok(result)
    }

    fn accept_provides(
        &self,
        node: &Node,
        module: &ModuleInstance,
        provides: &Provider,
    ) -> Result<ComponentSections, TokenStream> {
        let mut args = quote! {};
        for arg in provides.get_dependencies() {
            let arg_provider_name = arg.get_field_type().identifier();
            args = quote! {
                #args  self.#arg_provider_name(),
            }
        }

        let bound;
        if self.graph.has_scoped_deps(node)? {
            bound = "_";
        } else {
            bound = "";
        }
        let type_path = node.type_.syn_type_with_innner_lifetime_bound(bound);

        let name_ident = node.type_.identifier();
        let module_method = format_ident!("{}", provides.get_name());
        let invoke_module;

        if provides.get_field_static() {
            let module_path = module.type_.syn_type();
            invoke_module = quote! {#module_path::#module_method(#args)}
        } else {
            let module_name = module.name.clone();
            invoke_module = quote! {self.#module_name.#module_method(#args)}
        }
        let mut result = ComponentSections::new();
        result.add_methods(quote! {
            fn #name_ident(&'_ self) -> #type_path{
                #invoke_module
            }
        });
        Ok(result)
    }

    fn accept_binds(
        &self,
        node: &Node,
        _module: &ModuleInstance,
        provides: &Provider,
    ) -> Result<ComponentSections, TokenStream> {
        let arg = provides
            .get_dependencies()
            .first()
            .expect("binds must have one arg");
        let arg_provider_name = arg.get_field_type().identifier();

        let name_ident = node.type_.identifier();
        let bound;
        if self.graph.has_scoped_deps(node)? {
            bound = "_";
        } else {
            bound = "";
        }
        let type_path = node.type_.syn_type_with_innner_lifetime_bound(bound);

        let mut result = ComponentSections::new();
        result.add_methods(quote! {
            fn #name_ident(&'_ self) -> #type_path{
                Box::new(self.#arg_provider_name(),)
            }
        });
        Ok(result)
    }

    fn accept_boxed(
        &self,
        node: &Node,
        boxed_node: &Node,
    ) -> Result<ComponentSections, TokenStream> {
        let arg_provider_name = boxed_node.type_.identifier();
        let name_ident = node.type_.identifier();
        let bound;
        if self.graph.has_scoped_deps(node)? {
            bound = "_";
        } else {
            bound = "";
        }

        let type_path = node.type_.syn_type_with_innner_lifetime_bound(bound);

        let mut result = ComponentSections::new();
        result.add_methods(quote! {
            fn #name_ident(&'_ self) -> #type_path{
                Box::new(self.#arg_provider_name(),)
            }
        });
        Ok(result)
    }

    fn accept_scoped(
        &self,
        node: &Node,
        boxed_node: &Node,
    ) -> Result<ComponentSections, TokenStream> {
        let arg_provider_name = boxed_node.type_.identifier();
        let once_name = format_ident!("once_{}", node.type_.identifier());
        let once_type = boxed_node.type_.syn_type();

        let name_ident = node.type_.identifier();
        let type_path = node.type_.syn_type();
        let mut result = ComponentSections::new();
        result.add_fields(quote! {
            #once_name : lockjaw::Once<#once_type>,
        });
        result.add_ctor_params(quote! {#once_name : lockjaw::Once::new(),});
        result.add_methods(quote! {
            fn #name_ident(&'_ self) -> &#type_path{
                self.#once_name.get(|| self.#arg_provider_name())
            }
        });
        Ok(result)
    }
}

/// Creates human readable name for a node.
struct ProviderNameVisitor {}
impl NodeVisitor<String> for ProviderNameVisitor {
    fn accept_injectable(&self, node: &Node, _injectable: &Injectable) -> String {
        format!("{} (injectable)", node.type_.canonical_string_path())
    }

    fn accept_provides(
        &self,
        _node: &Node,
        module: &ModuleInstance,
        provides: &Provider,
    ) -> String {
        format!(
            "{}.{} (module provides)",
            module.type_.canonical_string_path(),
            provides.get_name()
        )
    }

    fn accept_binds(&self, _node: &Node, module: &ModuleInstance, provides: &Provider) -> String {
        format!(
            "{}.{} (module binds)",
            module.type_.canonical_string_path(),
            provides.get_name()
        )
    }

    fn accept_boxed(&self, node: &Node, _boxed_node: &Node) -> String {
        format!("{} (auto boxed)", node.type_.canonical_string_path())
    }

    fn accept_scoped(&self, node: &Node, _boxed_node: &Node) -> String {
        format!("ref {}", node.type_.canonical_string_path())
    }
}

/// Decides whether a node can have duplicates and merge them, for multibindings.
struct MergeNodeVisitor<'a> {
    new_node: &'a Node,
}

impl<'a> MergeNodeVisitor<'a> {
    fn duplicated<T>(&self, node: &Node) -> Result<T, TokenStream> {
        return compile_error(&format!(
            "found duplicated bindings for {}, provided by:\n\t{}\n\t{}",
            node.type_.canonical_string_path(),
            node.get_name(),
            self.new_node.get_name()
        ));
    }
}

impl<'a> NodeVisitor<Result<Node, TokenStream>> for MergeNodeVisitor<'a> {
    fn accept_injectable(
        &self,
        node: &Node,
        _injectable: &Injectable,
    ) -> Result<Node, TokenStream> {
        self.duplicated(node)
    }

    fn accept_provides(
        &self,
        node: &Node,
        _module: &ModuleInstance,
        _provides: &Provider,
    ) -> Result<Node, TokenStream> {
        self.duplicated(node)
    }

    fn accept_binds(
        &self,
        node: &Node,
        _module: &ModuleInstance,
        _provides: &Provider,
    ) -> Result<Node, TokenStream> {
        self.duplicated(node)
    }

    fn accept_boxed(&self, node: &Node, _boxed_node: &Node) -> Result<Node, TokenStream> {
        if node
            .type_
            .canonical_string_path()
            .eq(&self.new_node.type_.canonical_string_path())
        {
            return Ok(node.clone());
        }
        self.duplicated(node)
    }

    fn accept_scoped(&self, node: &Node, _boxed_node: &Node) -> Result<Node, TokenStream> {
        self.duplicated(node)
    }
}

/// Decide whether a node can depend on another node. Regular nodes cannot depend on scoped types.
struct CanDepend<'a> {
    target_node: &'a Node,
    ancestors: &'a Vec<String>,
}

impl<'a> CanDepend<'a> {
    fn no_scope(&self) -> Result<(), TokenStream> {
        compile_error(&format!(
            "unable to provide scoped binding as regular type {}\nrequested by:{}",
            self.target_node.get_name(),
            self.ancestors.join("\nrequested by:")
        ))
    }
}

impl<'a> NodeVisitor<Result<(), TokenStream>> for CanDepend<'a> {
    fn accept_injectable(&self, _node: &Node, _injectable: &Injectable) -> Result<(), TokenStream> {
        if self.target_node.scoped {
            return self.no_scope();
        }
        Ok(())
    }

    fn accept_provides(
        &self,
        _node: &Node,
        _module: &ModuleInstance,
        _provides: &Provider,
    ) -> Result<(), TokenStream> {
        if self.target_node.scoped {
            return self.no_scope();
        }
        Ok(())
    }

    fn accept_binds(
        &self,
        _node: &Node,
        _module: &ModuleInstance,
        _provides: &Provider,
    ) -> Result<(), TokenStream> {
        if self.target_node.scoped {
            return self.no_scope();
        }
        Ok(())
    }

    fn accept_boxed(&self, _node: &Node, _boxed_node: &Node) -> Result<(), TokenStream> {
        if self.target_node.scoped {
            return self.no_scope();
        }
        Ok(())
    }

    fn accept_scoped(&self, _node: &Node, _boxed_node: &Node) -> Result<(), TokenStream> {
        Ok(())
    }
}

fn get_module_manifest(
    manifest: &Manifest,
    component: &Component,
) -> Result<ComponentModuleManifest, TokenStream> {
    if !component.has_module_manifest() {
        return Ok(ComponentModuleManifest::new());
    }
    for module_manifest in manifest.get_component_module_manifests() {
        if module_manifest
            .get_field_type()
            .identifier()
            .eq(&component.get_module_manifest().identifier())
        {
            return Ok(module_manifest.clone());
        }
    }
    compile_error(&format!(
        "cannot find module manifest {}, used by {}",
        component.get_module_manifest().canonical_string_path(),
        component.get_field_type().canonical_string_path()
    ))
}

fn build_graph(manifest: &Manifest, component: &Component) -> Result<Graph, TokenStream> {
    let mut result = Graph::default();
    for injectable in manifest.get_injectables() {
        let _: Vec<()> = generate_injectable_nodes(injectable)
            .iter()
            .map(|node| result.add_node(node))
            .collect::<Result<Vec<()>, TokenStream>>()?;
    }
    let mut installed_modules = HashSet::<Ident>::new();
    result.module_manifest = get_module_manifest(manifest, component)?;

    for module in &result.module_manifest.modules {
        installed_modules.insert(module.identifier());
    }

    for module in result.module_manifest.get_builder_modules() {
        installed_modules.insert(module.get_field_type().identifier());
    }

    for module in manifest.get_modules() {
        if !installed_modules.contains(&module.get_field_type().identifier()) {
            continue;
        }
        for provider in module.get_providers() {
            let _ = generate_provides_nodes(
                &result.module_manifest,
                &module.get_field_type(),
                provider,
            )
            .iter()
            .map(|node| result.add_node(node))
            .collect::<Result<Vec<()>, TokenStream>>()?;
        }
    }
    Ok(result)
}

fn generate_injectable_nodes(injectable: &Injectable) -> Vec<Node> {
    let node = Node {
        type_: injectable.get_field_type().clone(),
        dependencies: injectable
            .fields
            .iter()
            .filter(|field| field.get_injected())
            .map(|field| field.get_field_type().clone())
            .collect(),
        node_type: NodeType::Injectable(injectable.clone()),
        scoped: false,
    };
    generate_node_variants(node)
}

fn get_module_instance(manifest: &ComponentModuleManifest, module_type: &Type) -> ModuleInstance {
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

fn generate_provides_nodes(
    module_manifest: &ComponentModuleManifest,
    module_type: &Type,
    provider: &Provider,
) -> Vec<Node> {
    let type_;
    let node_type;
    if provider.get_binds() {
        type_ = boxed_type(provider.get_field_type());
        node_type = NodeType::Binds(
            get_module_instance(module_manifest, module_type),
            provider.clone(),
        );
    } else {
        type_ = provider.get_field_type().clone();
        node_type = NodeType::Provides(
            get_module_instance(module_manifest, module_type),
            provider.clone(),
        );
    }
    let node = Node {
        type_,
        dependencies: provider
            .get_dependencies()
            .iter()
            .map(|dependency| dependency.get_field_type().clone())
            .collect(),
        node_type,
        scoped: false,
    };
    generate_node_variants(node)
}

fn generate_node_variants(node: Node) -> Vec<Node> {
    if !node.type_.get_scopes().is_empty() {
        let mut private_node = node.clone();
        private_node.scoped = true;

        let scoped_node = Node {
            type_: ref_type(&node.type_),
            dependencies: vec![private_node.type_.clone()],
            node_type: NodeType::Scoped(Box::new(private_node.clone())),
            scoped: false,
        };

        return vec![private_node, scoped_node];
    }

    if node.type_.scopes.is_empty() {
        if node.type_.get_path().ne("std::boxed::Box") {
            let boxed_node = Node {
                type_: boxed_type(&node.type_),
                dependencies: vec![node.type_.clone()],
                node_type: NodeType::Boxed(Box::new(node.clone())),
                scoped: false,
            };
            return vec![node, boxed_node];
        }
        return vec![node];
    }
    return vec![];
}

fn boxed_type(type_: &Type) -> Type {
    let mut boxed_type = Type::new();
    boxed_type.set_root(Type_Root::GLOBAL);
    boxed_type.set_path("std::boxed::Box".to_string());
    boxed_type.mut_args().push(type_.clone());
    boxed_type
}

fn ref_type(type_: &Type) -> Type {
    let mut ref_type = type_.clone();
    ref_type.set_field_ref(true);
    ref_type
}
