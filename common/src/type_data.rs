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

use serde::{Deserialize, Serialize};

use crate::environment;
use crate::manifest::TypeRoot;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

#[derive(Debug, Serialize, Deserialize, Clone, Default, Eq)]
pub struct TypeData {
    pub root: TypeRoot,
    pub path: String,
    pub field_crate: String,
    pub args: Vec<TypeData>,
    pub trait_object: bool,
    pub field_ref: bool,
    pub scopes: HashSet<TypeData>,
    pub identifier_suffix: String,
    pub qualifier: Option<Box<TypeData>>,
}

impl PartialEq for TypeData {
    fn eq(&self, other: &Self) -> bool {
        self.identifier_string().eq(&other.identifier_string())
    }
}

impl Hash for TypeData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.identifier_string().hash(state)
    }
}

impl TypeData {
    pub fn new() -> Self {
        Default::default()
    }

    /// Full path of the type in universal from ($CRATE always resolved)
    ///
    /// Modifiers like & are omitted
    pub fn canonical_string_path(&self) -> String {
        let prefix = self.get_prefix();
        match self.root {
            TypeRoot::GLOBAL => format!("{}::{}", prefix, self.path_with_args(false)),
            TypeRoot::CRATE => {
                format!(
                    "{}::{}::{}",
                    prefix,
                    self.field_crate,
                    self.path_with_args(false)
                )
            }
            TypeRoot::PRIMITIVE => format!("{}{}", prefix, self.path),
            TypeRoot::UNSPECIFIED => panic!("canonical_string_path: root unspecified"),
        }
    }

    pub fn canonical_string_path_without_args(&self) -> String {
        let prefix = self.get_prefix();
        match self.root {
            TypeRoot::GLOBAL => format!("{}::{}", prefix, self.path_with_args(false)),
            TypeRoot::CRATE => {
                format!("{}::{}::{}", prefix, self.field_crate, self.path)
            }
            TypeRoot::PRIMITIVE => format!("{}{}", prefix, self.path),
            TypeRoot::UNSPECIFIED => panic!("canonical_string_path: root unspecified"),
        }
    }

    /// Full path of the type in local from (use crate:: within the same crate).
    ///
    /// Modifiers like & are omitted
    pub fn local_string_path(&self) -> String {
        let prefix = self.get_prefix();
        match self.root {
            TypeRoot::GLOBAL => format!("{}::{}", prefix, self.path_with_args(true)),
            TypeRoot::CRATE => {
                if environment::current_crate().eq(&self.field_crate) {
                    format!("{}crate::{}", prefix, self.path_with_args(true))
                } else {
                    format!(
                        "{}{}::{}",
                        prefix,
                        self.field_crate,
                        self.path_with_args(true)
                    )
                }
            }
            TypeRoot::PRIMITIVE => format!("{}{}", prefix, self.path),
            TypeRoot::UNSPECIFIED => panic!("local_string_path: root unspecified"),
        }
    }

    fn get_prefix(&self) -> String {
        let mut prefix = String::new();
        if self.field_ref {
            prefix.push_str("& ");
        }
        if self.trait_object {
            prefix.push_str("dyn ");
        }
        prefix
    }

    /// Unique identifier token representing the type.
    ///
    /// Modifiers like & are included.
    pub fn identifier_string(&self) -> String {
        let prefix = self
            .qualifier
            .as_ref()
            .map(|qualifier| format!("ᑕ{}ᑐ_", qualifier.identifier_string()))
            .unwrap_or("".to_owned());
        format!(
            "{}{}_{}",
            prefix,
            self.canonical_string_path()
                .replace("::", "ⵆ")
                .replace("<", "ᐸ")
                .replace(">", "ᐳ")
                .replace(" ", "_")
                .replace("\'", "ᐠ")
                .replace("&", "ε")
                .replace(",", "ᒧ"),
            self.identifier_suffix
        )
    }

    /// Human readable form.
    pub fn readable(&self) -> String {
        let mut prefix = String::new();
        if self.qualifier.is_some() {
            prefix.push_str(
                &format! {"#[qualified({})] ", self.qualifier.as_ref().unwrap().readable()},
            );
        }
        if self.field_ref {
            prefix.push_str("ref ");
        }
        format!("{}{}", prefix, self.canonical_string_path())
    }

    fn path_with_args(&self, local: bool) -> String {
        if self.args.is_empty() {
            return self.path.clone();
        }
        let args = self
            .args
            .iter()
            .map(|t| {
                if local {
                    t.local_string_path()
                } else {
                    t.canonical_string_path()
                }
            })
            .collect::<Vec<String>>()
            .join(",");
        format!("{}<{}>", self.path, args)
    }
}
