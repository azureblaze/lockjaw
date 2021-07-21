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

use crate::error::compile_error;
use crate::graph::{ComponentSections, Graph};
use crate::manifest::{MultibindingMapKey, TypeRoot};
use crate::nodes::node::Node;
use crate::type_data::TypeData;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::iter::Extend;

#[derive(Debug, Clone)]
pub struct MapNode {
    pub type_: TypeData,
    pub bindings: HashMap<MultibindingMapKey, TypeData>,
}

impl MapNode {
    pub fn new(
        map_key: &MultibindingMapKey,
        value_type: &TypeData,
    ) -> Result<Box<MapNode>, TokenStream> {
        Ok(Box::new(MapNode {
            type_: map_type(&key_type(&map_key)?, value_type)?,
            bindings: HashMap::new(),
        }))
    }

    pub fn with_key_type(
        map_key: &TypeData,
        value_type: &TypeData,
    ) -> Result<Box<MapNode>, TokenStream> {
        Ok(Box::new(MapNode {
            type_: map_type(&map_key, value_type)?,
            bindings: HashMap::new(),
        }))
    }

    pub fn add_binding(
        &mut self,
        map_key: &MultibindingMapKey,
        value_type: &TypeData,
    ) -> &mut Self {
        self.bindings.insert(map_key.clone(), value_type.clone());
        self
    }
}

fn key_type(map_key: &MultibindingMapKey) -> Result<TypeData, TokenStream> {
    Ok(match map_key {
        MultibindingMapKey::String(_) => string_type(),
        MultibindingMapKey::I32(_) => i32_type(),
        MultibindingMapKey::Enum(ref enum_type, _) => enum_type.clone(),
        _ => return compile_error("unable to handle key"),
    })
}

fn map_type(key_type: &TypeData, value_type: &TypeData) -> Result<TypeData, TokenStream> {
    let mut map_type = TypeData::new();
    map_type.root = TypeRoot::GLOBAL;
    map_type.path = "std::collections::HashMap".to_string();
    map_type.args.push(key_type.clone());
    map_type.args.push(value_type.clone());
    map_type.qualifier = value_type.qualifier.clone();
    Ok(map_type)
}

fn string_type() -> TypeData {
    let mut string_type = TypeData::new();
    string_type.root = TypeRoot::GLOBAL;
    string_type.path = "std::string::String".to_string();
    string_type
}

fn i32_type() -> TypeData {
    let mut string_type = TypeData::new();
    string_type.root = TypeRoot::PRIMITIVE;
    string_type.path = "i32".to_string();
    string_type
}

impl Node for MapNode {
    fn get_name(&self) -> String {
        return format!("{} (multibinding)", self.type_.readable());
    }

    fn generate_implementation(&self, _graph: &Graph) -> Result<ComponentSections, TokenStream> {
        let name_ident = self.get_identifier();
        let provides_type = self.type_.syn_type();
        let mut into_maps = quote! {};
        for binding in &self.bindings {
            let key = match binding.0 {
                MultibindingMapKey::String(ref key) => {
                    quote! { #key.to_owned() }
                }
                MultibindingMapKey::I32(key) => {
                    quote! { #key }
                }
                MultibindingMapKey::Enum(_, value_type) => {
                    let key = value_type.syn_type();
                    quote! { #key }
                }
                _ => return compile_error(&format!("unable to handle key {:?}", binding.0)),
            };
            let ident = binding.1.identifier();
            into_maps = quote! {
                #into_maps
                result.insert(#key, self.#ident());
            }
        }

        let mut result = ComponentSections::new();
        result.add_methods(quote! {
            #[allow(unused_mut)]
            fn #name_ident(&'_ self) -> #provides_type{
                let mut result = HashMap::new();
                #into_maps
                result
            }
        });

        Ok(result)
    }

    fn merge(&self, new_node: &dyn Node) -> Result<Box<dyn Node>, TokenStream> {
        if new_node.type_id() != TypeId::of::<MapNode>() {
            return <dyn Node>::duplicated(self, new_node);
        }
        let map_node = new_node.as_any().downcast_ref::<MapNode>().unwrap();
        for key in map_node.bindings.keys() {
            if self.bindings.contains_key(key) {
                return compile_error(&format!(
                    "found duplicated key {:?} for {}, provided by:\n\t{}",
                    key,
                    self.type_.readable(),
                    new_node.get_name()
                ));
            }
        }
        let mut new_map = self.bindings.clone();
        new_map.extend(
            map_node
                .bindings
                .iter()
                .map(|(k, v)| (k.clone(), v.clone())),
        );
        Ok(Box::new(MapNode {
            type_: self.type_.clone(),
            bindings: new_map,
        }))
    }

    fn get_type(&self) -> &TypeData {
        &self.type_
    }

    fn get_identifier(&self) -> Ident {
        self.type_.identifier()
    }

    fn get_dependencies(&self) -> Vec<TypeData> {
        self.bindings
            .iter()
            .map(|binding| binding.1.clone())
            .collect()
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
