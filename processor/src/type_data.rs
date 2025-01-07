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

use lazy_static::lazy_static;
use lockjaw_common::type_data::TypeData;
use std::collections::{HashMap, HashSet};

lazy_static! {
    /// auto used types that does not need fully qualified paths.
    static ref PRELUDE_V1: HashMap<String, String> = {
        let mut m = HashMap::<String, String>::new();
        m.insert("Box".into(), "std::boxed::Box".into());
        m.insert("Option".into(), "std::option::Option".into());
        m.insert("Result".into(), "std::result::Result".into());
        m.insert("String".into(), "std::string::String".into());
        m.insert("Vec".into(), "std::vec::Vec".into());
        m.insert("Cl".into(),"lockjaw::Cl".into() );
        m.insert("Provider".into(),"lockjaw::Provider".into() );
        m.insert("Lazy".into(),"lockjaw::Lazy".into() );
        m
    };
}

lazy_static! {
    /// primitive data types with no path
    static ref PRIMITIVES: HashSet<String> = {
        let mut m = HashSet::<String>::new();
        m.insert("i8".to_owned());
        m.insert("u8".to_owned());
        m.insert("i16".to_owned());
        m.insert("u16".to_owned());
        m.insert("i32".to_owned());
        m.insert("u32".to_owned());
        m.insert("i64".to_owned());
        m.insert("u64".to_owned());
        m.insert("i128".to_owned());
        m.insert("u128".to_owned());
        m.insert("isize".to_owned());
        m.insert("usize".to_owned());
        m.insert("f32".to_owned());
        m.insert("f64".to_owned());
        m.insert("bool".to_owned());
        m.insert("char".to_owned());
        m
    };
}

pub trait ProcessorTypeData {
    /// Full path of the type in local from (use crate:: within the same crate), which can be
    /// converted to tokens.
    ///
    /// Modifiers like & are omitted
    fn syn_type(&self) -> syn::Type;
    /// Unique identifier token representing the type.
    ///
    /// Modifiers like & are included.
    fn identifier(&self) -> syn::Ident;
}
impl ProcessorTypeData for TypeData {
    /// Full path of the type in local from (use crate:: within the same crate), which can be
    /// converted to tokens.
    ///
    /// Modifiers like & are omitted
    fn syn_type(&self) -> syn::Type {
        syn::parse_str(&self.local_string_path()).expect(&format!(
            "cannot parse type path {}",
            self.local_string_path()
        ))
    }

    /// Unique identifier token representing the type.
    ///
    /// Modifiers like & are included.
    fn identifier(&self) -> syn::Ident {
        quote::format_ident!("{}", self.identifier_string())
    }
}
