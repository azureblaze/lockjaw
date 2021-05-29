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

use crate::manifest::BindingType::Provides;
use crate::manifest::TypeRoot::UNSPECIFIED;
use crate::type_data::TypeData;

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
    pub type_data: TypeData,
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
    pub type_data: TypeData,
    pub injected: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct Component {
    pub type_data: TypeData,
    pub provisions: Vec<Dependency>,
    pub module_manifest: Option<TypeData>,
}

impl Component {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct ComponentModuleManifest {
    pub type_data: Option<TypeData>,
    pub builder_modules: Vec<Dependency>,
    pub modules: Vec<TypeData>,
}

impl ComponentModuleManifest {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct Dependency {
    pub name: String,
    pub type_data: TypeData,
}

impl Dependency {
    pub fn new() -> Self {
        Default::default()
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
    pub type_data: TypeData,
    pub bindings: Vec<Binding>,
}

impl Module {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct Binding {
    // message fields
    pub name: String,
    pub type_data: TypeData,
    pub dependencies: Vec<Dependency>,
    pub field_static: bool,
    pub binding_type: BindingType,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum BindingType {
    Provides,
    Binds,
    BindsOptionOf,
}

impl Default for BindingType {
    fn default() -> Self {
        Provides
    }
}

impl Binding {
    pub fn new(binding_type: BindingType) -> Self {
        Binding {
            binding_type,
            field_static: true,
            ..Default::default()
        }
    }
}
