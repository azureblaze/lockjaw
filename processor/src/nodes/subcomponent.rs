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

use crate::component_visibles;
use crate::error::CompileError;
use crate::graph::{build_graph, ComponentSections, Graph};
use crate::manifest::ProcessorComponent;
use crate::nodes::component_lifetime::ComponentLifetimeNode;
use crate::nodes::map::MapNode;
use crate::nodes::node::{DependencyData, Node};
use crate::nodes::vec::VecNode;
use crate::type_data::ProcessorTypeData;
use lockjaw_common::manifest::{Component, Manifest, MultibindingType};
use lockjaw_common::type_data::TypeData;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::any::Any;

#[derive(Debug, Clone)]
pub struct SubcomponentNode {
    pub type_: TypeData,
    pub builder_type: TypeData,
    pub dependencies: Vec<DependencyData>,
    pub token_stream: TokenStream,
}

impl SubcomponentNode {
    pub fn new(
        manifest: &Manifest,
        component_type: &TypeData,
        parent_component_type: &TypeData,
        parent_multibinding_nodes: &Vec<Box<dyn Node>>,
    ) -> Result<Vec<Box<dyn Node>>, TokenStream> {
        let subcomponent = find_component(manifest, component_type).map_compile_error(&format!(
            "unable to find component {}",
            component_type.readable()
        ))?;
        let mut builder_type = component_type.clone();
        builder_type.path.push_str("Builder");
        builder_type.trait_object = true;
        let type_ = ComponentLifetimeNode::component_lifetime_type(&builder_type);
        let (graph, missing_deps) =
            build_graph(manifest, &subcomponent, parent_multibinding_nodes)?;

        let mut nodes: Vec<Box<dyn Node>> = Vec::new();
        nodes.push(Box::new(SubcomponentNode {
            type_,
            builder_type: builder_type.clone(),
            dependencies: missing_deps
                .iter()
                .map(|md| DependencyData {
                    type_: md.type_data.clone(),
                    message: md.to_message(),
                })
                .collect(),
            token_stream: generate_component(
                manifest,
                &subcomponent,
                &graph,
                parent_component_type,
                &builder_type,
            )?,
        }));
        for dep in missing_deps.iter() {
            match dep.multibinding_type {
                MultibindingType::IntoVec => nodes.push(VecNode::new(&dep.type_data.args[0])),
                MultibindingType::IntoMap => nodes.push(MapNode::with_key_type(
                    &dep.type_data.args[0],
                    &dep.type_data.args[1],
                )?),
                _ => {}
            }
        }

        Ok(nodes)
    }
}

fn generate_component(
    manifest: &Manifest,
    component: &Component,
    graph: &Graph,
    parent_component_type: &TypeData,
    builder_type: &TypeData,
) -> Result<TokenStream, TokenStream> {
    let component_name =
        component_visibles::visible_type(graph.manifest, &component.type_data).syn_type();
    let component_impl_name = component.impl_ident();

    let component_builder_impl_name = format_ident!("SubcomponentBuilderImpl",);

    let mut component_sections = ComponentSections::new();

    component_sections.merge(graph.generate_modules(manifest));
    component_sections.merge(graph.generate_provisions(component)?);

    let fields = &component_sections.fields;
    let ctor_params = &component_sections.ctor_params;
    let ctor_statements = &component_sections.ctor_statements;
    let methods = &component_sections.methods;
    let trait_methods = &component_sections.trait_methods;
    let items = &component_sections.items;
    let parent_impl_type = format_ident!("{}Impl", parent_component_type.identifier().to_string());

    let mut builder_type_without_dyn =
        component_visibles::visible_type(graph.manifest, &builder_type).clone();
    builder_type_without_dyn.trait_object = false;
    let builder_syn_type = builder_type_without_dyn.syn_type();

    let builder_param = if let Some(ref builder_modules) = component.builder_modules {
        let param_type =
            component_visibles::visible_type(graph.manifest, &builder_modules).syn_type();
        quote! {param: #param_type}
    } else {
        quote! {}
    };

    let component_impl = quote! {
        #[allow(non_snake_case)]
        #[allow(non_camel_case_types)]
        #[allow(dead_code)]
        struct #component_impl_name<'a> {
            parent: &'a #parent_impl_type,
            #fields
        }
        #[allow(non_snake_case)]
        impl <'a> #component_impl_name<'a> {
            #methods
        }
        #[allow(non_snake_case)]
        impl <'a> #component_name<'a> for #component_impl_name<'a> {
            #trait_methods
        }

        #items

        #[allow(non_snake_case)]
        #[allow(non_camel_case_types)]
        struct #component_builder_impl_name<'a> {
            parent: &'a #parent_impl_type,
        }

        #[allow(non_snake_case)]
        impl <'a> #builder_syn_type<'a> for #component_builder_impl_name<'a> {

            fn build(&self, #builder_param) -> lockjaw::Cl<'a, dyn #component_name<'a>> {
                #ctor_statements
                lockjaw::Cl::Val(::std::boxed::Box::new(#component_impl_name{parent: self.parent, #ctor_params}))
            }
        }

        lockjaw::Cl::Val(::std::boxed::Box::new(#component_builder_impl_name {parent: self}))
    };

    Ok(quote! {
        #component_impl
    })
}

fn find_component(manifest: &Manifest, component_type: &TypeData) -> Option<Component> {
    let identifier = component_type.identifier();
    for component in &manifest.components {
        if component.type_data.identifier() == identifier {
            return Some(component.clone());
        }
    }
    return None;
}

impl Node for SubcomponentNode {
    fn get_name(&self) -> String {
        format!("{} (subcomponent builder)", self.type_.readable())
    }

    fn generate_implementation(&self, graph: &Graph) -> Result<ComponentSections, TokenStream> {
        let mut component_sections = ComponentSections::new();

        let name_ident = self.get_identifier();
        let type_path =
            component_visibles::visible_type(graph.manifest, &self.builder_type).syn_type();

        let impl_tokens = self.token_stream.clone();

        component_sections.add_methods(quote! {
            fn #name_ident(&'_ self) -> ::lockjaw::Cl<'_, #type_path>{
                #impl_tokens
            }
        });

        Ok(component_sections)
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
        self.dependencies.clone()
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
