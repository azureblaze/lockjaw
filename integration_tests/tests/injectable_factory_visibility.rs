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

#![allow(dead_code)]

use lockjaw::{component, component_visible, epilogue, injectable};

#[component_visible]
struct Private {
    pub i: i32,
}

#[injectable]
impl Private {
    #[factory]
    fn create(#[runtime] i: i32) -> Self {
        Self { i }
    }
}

#[derive(Debug)]
pub struct Public {
    pub i: i32,
}

#[injectable]
impl Public {
    #[factory(visibility: "pub")]
    fn create(#[runtime] i: i32) -> Self {
        Self { i }
    }
}

#[component_visible]
pub(crate) struct CratePublic {
    pub i: i32,
}

#[injectable]
impl CratePublic {
    #[factory(visibility: "pub(crate)")]
    fn create(#[runtime] i: i32) -> Self {
        Self { i }
    }
}

#[component]
pub trait MyComponent {
    fn private_factory(&self) -> PrivateFactory;
    fn public_factory(&self) -> PublicFactory;
    fn crate_public_factory(&self) -> CratePublicFactory;
}

#[test]
pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::build();

    assert_eq!(component.private_factory().create(42).i, 42);
    assert_eq!(component.public_factory().create(42).i, 42);
    assert_eq!(component.crate_public_factory().create(42).i, 42);
}

epilogue!();
