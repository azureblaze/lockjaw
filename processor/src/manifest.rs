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

use crate::environment;
use crate::manifest::TypeRoot::UNSPECIFIED;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct Manifest {
    pub injectables: Vec<Injectable>,
    pub components: Vec<Component>,
    pub merged_crates: Vec<::std::string::String>,
    pub modules: Vec<Module>,
    pub component_module_manifests: Vec<ComponentModuleManifest>,
}

impl Manifest {
    pub fn new() -> Manifest {
        Default::default()
    }

    pub fn clear(&mut self) {
        self.injectables.clear();
        self.components.clear();
        self.merged_crates.clear();
        self.modules.clear();
        self.component_module_manifests.clear();
    }

    pub fn merge_from(&mut self, other: &Manifest) {
        self.injectables
            .extend_from_slice(other.injectables.as_slice());
        self.components
            .extend_from_slice(other.components.as_slice());
        self.merged_crates
            .extend_from_slice(other.merged_crates.as_slice());
        self.modules.extend_from_slice(other.modules.as_slice());
        self.component_module_manifests
            .extend_from_slice(other.component_module_manifests.as_slice());
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct Injectable {
    pub field_type: Type,
    pub ctor_name: String,
    pub dependencies: Vec<Dependency>,
}

impl Injectable {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct Field {
    pub name: String,
    pub field_type: Type,
    pub injected: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct Component {
    pub field_type: Type,
    pub provisions: Vec<Dependency>,
    pub module_manifest: Option<Type>,
}

impl Component {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct ComponentModuleManifest {
    pub field_type: Option<Type>,
    pub builder_modules: Vec<Dependency>,
    pub modules: Vec<Type>,
}

impl ComponentModuleManifest {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct Dependency {
    pub name: String,
    pub field_type: Type,
}

impl Dependency {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct Type {
    pub root: TypeRoot,
    pub path: String,
    pub field_crate: String,
    pub args: Vec<Type>,
    pub trait_object: bool,
    pub field_ref: bool,
    pub scopes: Vec<Type>,
}

impl Type {
    pub fn new() -> Self {
        Default::default()
    }

    /// Full path of the type in universal from ($CRATE always resolved)
    ///
    /// Modifiers like & are omitted
    pub fn canonical_string_path(&self) -> String {
        match self.root {
            TypeRoot::GLOBAL => format!("::{}", self.path_with_args()),
            TypeRoot::CRATE => {
                format!("::{}::{}", self.field_crate, self.path_with_args())
            }
            TypeRoot::PRIMITIVE => format!("{}", self.path),
            TypeRoot::UNSPECIFIED => panic!("canonical_string_path: root unspecified"),
        }
    }

    /// Full path of the type in local from (use crate:: within the same crate).
    ///
    /// Modifiers like & are omitted
    pub fn local_string_path(&self) -> String {
        match self.root {
            TypeRoot::GLOBAL => format!("::{}", self.path_with_args()),
            TypeRoot::CRATE => {
                if environment::current_crate().eq(&self.field_crate) {
                    format!("crate::{}", self.path_with_args())
                } else {
                    format!("{}::{}", self.field_crate, self.path_with_args())
                }
            }
            TypeRoot::PRIMITIVE => format!("{}", self.path),
            TypeRoot::UNSPECIFIED => panic!("local_string_path: root unspecified"),
        }
    }

    /// Full path of the type in local from (use crate:: within the same crate), which can be
    /// converted to tokens.
    ///
    /// Modifiers like & are omitted
    pub fn syn_type(&self) -> syn::Type {
        syn::parse_str(&self.local_string_path()).expect("cannot parse type path")
    }

    /// Unique identifier token representing the type.
    ///
    /// Modifiers like & are included.
    pub fn identifier(&self) -> syn::Ident {
        let mut prefix = String::new();
        if self.field_ref {
            prefix.push_str("ref_");
        }
        quote::format_ident!(
            "{}{}",
            prefix,
            self.canonical_string_path()
                .replace("::", "_")
                .replace("<", "_L_")
                .replace(">", "_R_")
                .replace(" ", "_")
                .replace("\'", "")
        )
    }

    /// Human readable form.
    pub fn readable(&self) -> String {
        let mut prefix = String::new();
        if self.field_ref {
            prefix.push_str("ref ");
        }
        format!("{}{}", prefix, self.canonical_string_path())
    }

    fn path_with_args(&self) -> String {
        let prefix = if self.trait_object { "dyn " } else { "" };
        if self.args.is_empty() {
            return format!("{}{}", prefix, self.path);
        }
        let args = self
            .args
            .iter()
            .map(|t| t.path_with_args())
            .collect::<Vec<String>>()
            .join(",");
        format!("{}{}<{}>", prefix, self.path, args)
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum TypeRoot {
    UNSPECIFIED = 0,
    GLOBAL = 1,
    CRATE = 2,
    PRIMITIVE = 3,
}

impl Default for TypeRoot {
    fn default() -> Self {
        UNSPECIFIED
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct Module {
    pub field_type: Type,
    pub providers: Vec<Provider>,
}

impl Module {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct Provider {
    // message fields
    pub name: String,
    pub field_type: Type,
    pub dependencies: Vec<Dependency>,
    pub field_static: bool,
    pub binds: bool,
}

impl Provider {
    pub fn new() -> Self {
        Provider {
            field_static: true,
            ..Default::default()
        }
    }
}
