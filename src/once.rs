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

use std::cell::RefCell;
use std::mem::MaybeUninit;

/// once
#[doc(hidden)]
pub struct Once<T> {
    once: std::sync::Once,
    value: RefCell<MaybeUninit<T>>,
}

impl<T> Once<T> {
    pub fn new() -> Self {
        Once {
            once: std::sync::Once::new(),
            value: RefCell::new(MaybeUninit::uninit()),
        }
    }

    pub fn get<F>(&self, initializer: F) -> &T
    where
        F: FnOnce() -> T,
    {
        unsafe {
            let value = &self.value;
            self.once.call_once(|| {
                value.borrow_mut().as_mut_ptr().write(initializer());
            });
            &*value.borrow().as_ptr()
        }
    }
}
