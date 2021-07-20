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

use crate::graph::{ComponentSections, Graph};
use crate::nodes::node::Node;
use crate::type_data::TypeData;
use proc_macro2::TokenStream;
use quote::quote;
use std::any::Any;

#[derive(Debug, Clone)]
pub struct ParentNode {
    pub type_: TypeData,
}

impl ParentNode {
    pub fn new(type_: &TypeData) -> Result<Box<Self>, TokenStream> {
        Ok(Box::new(ParentNode {
            type_: type_.clone(),
        }))
    }
}

impl Node for ParentNode {
    fn get_name(&self) -> String {
        format!("{} (parent component access)", self.type_.readable())
    }

    fn generate_implementation(&self, _graph: &Graph) -> Result<ComponentSections, TokenStream> {
        let name_ident = self.get_identifier();
        let syn_type = self.type_.syn_type();

        let mut result = ComponentSections::new();

        result.add_methods(quote! {
            fn #name_ident(&'_ self) -> #syn_type{
                self.parent.#name_ident()
            }
        });

        Ok(result)
    }

    fn get_type(&self) -> &TypeData {
        &self.type_
    }

    fn clone_box(&self) -> Box<dyn Node> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
