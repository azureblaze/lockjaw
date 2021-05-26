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

use crate::{Once, Provider};

pub struct Lazy<'a, T> {
    provider: Provider<'a, T>,
    value: Once<T>,
}

impl<'a, T> Lazy<'a, T> {
    pub fn new(provider: Provider<'a, T>) -> Self {
        Lazy {
            provider,
            value: Once::new(),
        }
    }

    pub fn get(&'a self) -> &'a T {
        self.value.get(|| self.provider.get())
    }
}
