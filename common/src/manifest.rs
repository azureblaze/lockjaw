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
use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct Manifest {
    pub injectables: Vec<Injectable>,
    pub components: Vec<Component>,
    pub merged_crates: Vec<String>,
    pub modules: Vec<Module>,
    pub builder_modules: Vec<BuilderModules>,
    pub qualifiers: Vec<TypeData>,
    pub entry_points: Vec<EntryPoint>,
    pub root: bool,
    pub expanded_visibilities: HashMap<String, ExpandedVisibility>,
    pub lifetimed_types: HashSet<TypeData>,
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
        self.builder_modules.clear();
        self.qualifiers.clear();
        self.entry_points.clear();
        self.root = false;
        self.expanded_visibilities.clear();
        self.lifetimed_types.clear();
    }

    pub fn merge_from(&mut self, other: &Manifest) {
        self.injectables
            .extend_from_slice(other.injectables.as_slice());
        self.components
            .extend_from_slice(other.components.as_slice());
        self.merged_crates
            .extend_from_slice(other.merged_crates.as_slice());
        self.modules.extend_from_slice(other.modules.as_slice());
        self.builder_modules
            .extend_from_slice(other.builder_modules.as_slice());
        self.qualifiers
            .extend_from_slice(other.qualifiers.as_slice());
        self.entry_points
            .extend_from_slice(other.entry_points.as_slice());
        self.expanded_visibilities.extend(
            other
                .expanded_visibilities
                .iter()
                .map(|(k, v)| (k.clone(), v.clone())),
        );
        self.lifetimed_types
            .extend(other.lifetimed_types.iter().map(Clone::clone));
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct Injectable {
    pub type_data: TypeData,
    pub ctor_name: String,
    pub dependencies: Vec<Dependency>,
    pub container: Option<TypeData>,
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

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum ComponentType {
    Component,
    Subcomponent,
}

impl Default for ComponentType {
    fn default() -> Self {
        ComponentType::Component
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct Component {
    pub name: String,
    pub type_data: TypeData,
    pub component_type: ComponentType,
    pub provisions: Vec<Dependency>,
    pub builder_modules: Option<TypeData>,
    pub modules: Vec<TypeData>,
    pub definition_only: bool,
    pub address: TypeData,
}

impl Component {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct EntryPoint {
    pub type_data: TypeData,
    pub component: TypeData,
    pub provisions: Vec<Dependency>,
    pub address: TypeData,
}

impl EntryPoint {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct BuilderModules {
    pub type_data: Option<TypeData>,
    pub builder_modules: Vec<Dependency>,
}

impl BuilderModules {
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

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Eq, Hash)]
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
    pub subcomponents: HashSet<TypeData>,
    pub install_in: HashSet<TypeData>,
}

impl Module {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct Binding {
    pub name: String,
    pub type_data: TypeData,
    pub dependencies: Vec<Dependency>,
    pub field_static: bool,
    pub binding_type: BindingType,
    pub multibinding_type: MultibindingType,
    pub map_key: MultibindingMapKey,
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

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum BindingType {
    Provides,
    Binds,
    BindsOptionOf,
    Multibinds,
}

impl Default for BindingType {
    fn default() -> Self {
        Provides
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum MultibindingType {
    None,
    IntoVec,
    ElementsIntoVec,
    IntoMap,
}

impl Default for MultibindingType {
    fn default() -> Self {
        MultibindingType::None
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Hash, Eq)]
pub enum MultibindingMapKey {
    None,
    String(String),
    I32(i32),
    Enum(TypeData, TypeData),
}

impl Default for MultibindingMapKey {
    fn default() -> Self {
        MultibindingMapKey::None
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Hash, Eq)]
pub struct ExpandedVisibility {
    pub exported_name: TypeData,
    pub crate_local_name: TypeData,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CfgManifest {
    pub prod_manifest: Manifest,
    pub test_manifest: Manifest,
}

impl CfgManifest {
    pub fn merge_from(&mut self, other: &CfgManifest) {
        self.prod_manifest.merge_from(&other.prod_manifest);
        self.test_manifest.merge_from(&other.test_manifest);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DepManifests {
    pub crate_name: String,
    pub prod_manifest: Vec<Manifest>,
    pub test_manifest: Vec<Manifest>,
    pub root_manifests: HashMap<String, CfgManifest>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LockjawPackage {
    pub id: String,
    pub name: String,
    pub src_path: String,
    pub direct_prod_crate_deps: Vec<String>,
    pub direct_test_crate_deps: Vec<String>,
}
