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

/// Wraps a binding so it can be lazily created.
///
/// When `Foo` depends on `Bar`, `Bar` is created before `Foo` is created which may not be desirable
/// if creating `Bar` is costly but it will only be used much later or only conditionally. By
/// wrapping `Bar` as `Lazy<Bar>`, `Bar` will only be created when [`Lazy<bar>.get()`](#method.get) is
/// called.
///
/// [`Lazy.get()`](#method.get) is cached and the same instance will be returned if called multiple times.
///
/// If multiple instances of the object is needed, use [`Provider<T>`](Provider) instead
///
/// ```
/// # use lockjaw::{epilogue, injectable, module, component, Cl, Lazy};
/// # use std::cell::RefCell;
/// # lockjaw::prologue!("src/lib.rs");
/// pub struct Counter {
///     counter: i32,
/// }
///
/// # #[injectable(scope: crate::MyComponent, container: RefCell)]
/// # impl Counter {
/// #    #[inject]
/// #    pub fn new() -> Self {
/// #        Self {
/// #            counter: 0,
/// #        }
/// #    }
/// #
/// #    pub fn get(&self) -> i32 {
/// #        self.counter
/// #    }
///#
/// #    pub fn increment(&mut self) -> i32 {
/// #        self.counter += 1;
/// #        self.counter
/// #    }
/// # }
/// pub struct Foo {
///     pub i: i32,
/// }
///
/// #[injectable]
/// impl Foo {
///     #[inject]
///     pub fn new(counter: &RefCell<Counter>) -> Foo{
///         Foo {
///             i: counter.borrow_mut().increment(),
///         }
///     }
/// }
///
/// #[component]
/// pub trait MyComponent {
///     fn foo(&self) -> Lazy<crate::Foo>;
///
///     fn counter(&self) -> &RefCell<Counter>;
/// }
///
/// pub fn main() {
///     let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
///     let counter = component.counter();
///     let lazy_foo = component.foo();
///     // Foo not created yet.
///     assert_eq!(counter.borrow().get(), 0);
///     
///     let foo = lazy_foo.get();
///     assert_eq!(counter.borrow().get(), 1);
///
///     // Same instance is reused
///     let foo2 = lazy_foo.get();
///     assert_eq!(foo.i, foo2.i);
/// }
/// epilogue!();
/// ```
pub struct Lazy<'a, T> {
    provider: Provider<'a, T>,
    value: Once<T>,
}

impl<'a, T> Lazy<'a, T> {
    #[doc(hidden)]
    pub fn new(provider: Provider<'a, T>) -> Self {
        Lazy {
            provider,
            value: Once::new(),
        }
    }

    /// Creates or retrieves a cached instance and returns a reference to it.
    pub fn get(&'a self) -> &'a T {
        self.value.get(|| self.provider.get())
    }
}
