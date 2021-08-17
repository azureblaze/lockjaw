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

/// Creates a binding on demand
///
/// `T` will be created each time [`Provider.get()`](#method.get) is called, allowing multiple
/// instances to be created.
///
/// This also prevents creating `T` immediately when constructing an object that depends on `T`,
/// which allows lazy initialization and can break cyclic dependency.
///
/// WARNING: calling `Provider.get()` in a constructor can lead to stackoverflow, and is best
/// avoided.
///
/// If only a single cached instance is needed, consider using `Lazy<T>`(Lazy) instead.
pub struct Provider<'a, T> {
    f: Box<dyn Fn() -> T + 'a>,
}

impl<'a, T> Provider<'a, T> {
    pub fn new(f: impl Fn() -> T + 'a) -> Self {
        Provider {
            f: std::boxed::Box::new(f),
        }
    }

    pub fn get(&self) -> T {
        (self.f)()
    }
}
