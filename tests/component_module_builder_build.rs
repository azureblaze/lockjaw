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

use lockjaw::{component, component_module_manifest, epilogue, module};

pub struct Foo {}

pub struct MyModule {
    string: String,
}
#[module]
impl MyModule {
    #[provides]
    pub fn provide_foo(&self) -> String {
        self.string.clone()
    }
}

#[component_module_manifest]
pub struct MyModuleManifest {
    #[builder]
    my_module: crate::MyModule,
}

#[component(modules = "crate::MyModuleManifest")]
pub trait MyComponent {
    fn string(&self) -> String;
}
#[test]
pub fn main() {
    let component: Box<dyn MyComponent> = MyComponent::build(MyModuleManifest {
        my_module: MyModule {
            string: "foo".to_owned(),
        },
    });
    assert_eq!(component.string(), "foo");
}
epilogue!();
