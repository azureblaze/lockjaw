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

use lockjaw::{
    component, component_module_manifest, injectable, module, module_impl, test_epilogue,
    MaybeScoped,
};

#[injectable(scope = "crate::MyComponent")]
pub struct Foo {}

pub trait MyTrait {
    fn hello(&self) -> String;
}

#[injectable]
pub struct MyTraitImpl<'a> {
    #[inject]
    foo: &'a crate::Foo,
}

impl MyTrait for MyTraitImpl<'_> {
    fn hello(&self) -> String {
        "hello".to_owned()
    }
}

#[module]
pub struct MyModule {}
#[module_impl]
impl MyModule {
    #[binds]
    pub fn bind_my_trait(_impl: crate::MyTraitImpl) -> impl crate::MyTrait {}
}

#[component_module_manifest]
pub struct MyModuleManifest {
    my_module: crate::MyModule,
}

#[component(modules = "crate::MyModuleManifest")]
pub trait MyComponent {
    fn my_trait(&'_ self) -> MaybeScoped<'_, dyn crate::MyTrait>;
}
#[test]
pub fn main() {
    let component: Box<dyn MyComponent> = MyComponent::new();
    assert_eq!(component.my_trait().hello(), "hello");
}
test_epilogue!();