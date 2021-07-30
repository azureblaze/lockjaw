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
use std::ops::Deref;

/// "Component Lifetime". Wrapper around an injection that may be scoped(owned by the component) or free standing(owned by
/// the item injecting it). Deref to access the content.
///
/// Typically this is used when the dependent does not care who owns the dependency, as it will
/// not try to move it. Injecting scoped dependency as 'T' or injected free standing dependency as
/// '&T' is a compile failure, but both can be injected as 'Cl<T>'
///
/// # Lifetime
///
/// 'Cl'\'s lifetime is bounded by the component providing it.
pub enum Cl<'a, T: ?Sized + 'a> {
    Val(Box<T>),
    Ref(&'a T),
}

impl<T: ?Sized> Deref for Cl<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Cl::Val(val) => val.deref(),
            Cl::Ref(r) => r,
        }
    }
}
