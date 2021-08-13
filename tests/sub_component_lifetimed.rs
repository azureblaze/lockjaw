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
use lockjaw::{component, module, prologue, subcomponent, injectable, Cl};

prologue!("tests/sub_component.rs");

pub struct Scoped {}

#[injectable(scope: MySubcomponent)]
impl Scoped {
    #[inject]
    pub fn new() -> Scoped {
        Scoped {}
    }
}

pub struct ScopedWithScopeDep<'a> {
    pub scoped: &'a Scoped,
}

#[injectable(scope: MySubcomponent)]
impl ScopedWithScopeDep<'_> {
    #[inject]
    pub fn new(scoped: &Scoped) -> ScopedWithScopeDep {
        ScopedWithScopeDep {
            scoped
        }
    }
}


#[subcomponent]
pub trait MySubcomponent<'a> {
    fn s(&self) -> &ScopedWithScopeDep;
}

struct MyModule {}

#[module(subcomponents: [MySubcomponent])]
impl MyModule {}

#[component(modules: [MyModule])]
pub trait MyComponent {
    fn sub(&'_ self) -> Cl<dyn MySubcomponentBuilder<'_>>;
}

#[test]
pub fn parent_binding() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    let sub: Cl<dyn MySubcomponent> = component.sub().build();

    let _ = sub.s().scoped;
}
lockjaw::epilogue!();
