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
use lockjaw;

#[lockjaw::component_visible]
struct DepPrivate {}

#[lockjaw::injectable]
impl DepPrivate {
    #[inject]
    pub fn new() -> Self {
        Self {}
    }
}

#[lockjaw::component_visible]
trait DepTrait {}

impl DepTrait for DepPrivate {}

pub struct DepInjectable {}

#[lockjaw::injectable]
impl DepInjectable {
    #[inject]
    pub fn new(_p: DepPrivate, _t: Cl<dyn DepTrait>) -> Self {
        Self {}
    }
}

pub struct DepProvided {}

#[lockjaw::component_visible]
struct DepModule {}

#[lockjaw::module(install_in: ::lockjaw::Singleton)]
impl DepModule {
    #[provides]
    pub fn provides_dep_provided(_p: DepPrivate) -> DepProvided {
        DepProvided {}
    }

    #[binds]
    pub fn bind_dep_trait(_impl: DepPrivate) -> Cl<dyn DepTrait> {}
}

#[lockjaw::component(modules: DepModule)]
pub trait DepComponent {
    fn dep(&self) -> crate::DepInjectable;
}

#[lockjaw::define_component]
pub trait DepDefinedComponent {}

#[lockjaw::entry_point(install_in: DepDefinedComponent)]
trait DepEntryPoint {
    fn dep(&self) -> crate::DepInjectable;
}

use lockjaw::Cl;
#[allow(unused_imports)]
use DepEntryPoint as DEP;
