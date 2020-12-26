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

#![cfg_attr(nightly, feature(external_doc))]

use std::cell::RefCell;
use std::mem::MaybeUninit;
use std::ops::Deref;

/// Annotates a trait that composes the dependency graph and provides items in the graph
/// (An "injector").
///
/// ```
/// # #[macro_use] extern crate lockjaw_processor;
/// # #[injectable]
/// # struct Foo{}
/// #
/// #[component]
/// trait MyComponent {
///     fn foo(&self) -> crate::Foo;
/// }
///
/// pub fn main() {
///     let component: Box<dyn MyComponent> = MyComponent::new();
///     let foo : Foo = component.foo();
/// }
/// # test_epilogue!();
/// ```
///
/// # Creating injected types
///
/// A component can declare methods to allow injected types to be created for code outside the
/// dependency graph. the method should take only `&self` as parameter, and return the injected
/// type.
///
/// Methods in a component must take only `&self` as parameter, and return a injected type. If the
/// returned type is not injected compilation will fail.
///
/// See [`injectable`] and [`module`] for how to make a type injectable.
///
/// Most types used by lockjaw must be fully qualified, i.e. it must start with either `::` or
/// `crate::`. The only expections are types included in the rust [prelude](std::prelude):
/// *   [Box]
/// *   [Option]
/// *   [Result]
/// *   [String]
/// *   [Vec]
///
/// lockjaw will complain non-fully qualified type at compile time
///
/// ```compile_fail
/// # #[macro_use] extern crate lockjaw_processor;
/// # #[injectable]
/// # struct Foo{}
/// #[component]
/// trait MyComponent {
///     fn foo(&self) -> Foo;
/// }
///
/// # fn main(){}
/// # test_epilogue!();
/// ```
/// # Installing modules
/// Each component can install their separate set of [`modules`](module) to form a different
/// dependency graph. Modules should be specified in a struct witht the [`component_module_manifest`]
/// attribute, and passed to `modules` in the `component` attribute.
///
/// ```
/// # #[macro_use] extern crate lockjaw_processor;
/// # #[module]
/// # struct StringModule {}
/// # #[module_impl]
/// # impl StringModule {
/// #     #[provides]
/// #     pub fn provide_string() -> String {
/// #         "string".to_owned()
/// #     }
/// # }
/// #
/// # #[module]
/// # struct UnsignedModule {}
/// # #[module_impl]
/// # impl UnsignedModule {
/// #     #[provides]
/// #     pub fn provide_unsigned() -> u32 {
/// #         42
/// #     }
/// # }
/// #
///
/// #[component_module_manifest]
/// struct MyModuleManifest {
///     string : crate::StringModule,
///     unsigned : crate::UnsignedModule
/// }
/// #[component(modules = "crate::MyModuleManifest")]
/// trait MyComponent {
///     fn string(&self) -> String;
///     fn unsigned(&self) -> u32;
/// }
///
/// # fn main() {}
/// # test_epilogue!();
/// ```
///
/// Component can select different modules providing the same type to change the behavior of types
/// that depend on it.
///
/// ```
/// # #[macro_use] extern crate lockjaw_processor;
///
/// #[injectable]
/// struct Foo{
///     #[inject]
///     string: String
/// }
///
/// # #[module]
/// # struct MyModule {}
/// #[module_impl]
/// impl MyModule {
///     #[provides]
///     pub fn provide_string() -> String { "string".to_owned() }
/// }
///
/// # #[module]
/// # struct OtherModule {}
/// #[module_impl]
/// impl OtherModule {
///     #[provides]
///     pub fn provide_string() -> String {"other_string".to_owned() }
/// }
///
/// #[component_module_manifest]
/// struct MyModuleManifest {
///     my_module : crate::MyModule
/// }
///
/// #[component(modules = "crate::MyModuleManifest")]
/// trait MyComponent {
///     fn foo(&self) -> crate::Foo;
/// }
///
/// #[component_module_manifest]
/// struct OtherModuleManifest {
///     other_module : crate::OtherModule
/// }
///
/// #[component(modules = "crate::OtherModuleManifest")]
/// trait OtherComponent {
///     fn foo(&self) -> crate::Foo;
/// }
///
/// pub fn main() {
///     let component: Box<dyn MyComponent> = MyComponent::new();
///     assert_eq!(component.foo().string, "string");
///     let other_component: Box<dyn OtherComponent> = OtherComponent::new();
///     assert_eq!(other_component.foo().string, "other_string");
/// }
/// # test_epilogue!();
/// ```
///
/// # Creating component instances
///
/// Lockjaw generates `COMPONENT::build(param: COMPONENT_MODULE_MANIFEST) -> Box<dyn COMPONENT>`,
/// which takes instances of modules and create the component.
///
/// ```
/// # #[macro_use] extern crate lockjaw_processor;
/// #[module]
/// pub struct StringModule {
///     string: String,
/// }
///
/// #[module_impl]
/// impl StringModule {
///     #[provides]
///     pub fn provide_string(&self) -> String {
///         self.string.clone()
///     }
/// }
///
/// #[component_module_manifest]
/// pub struct MyComponentModules {
///     #[builder]
///     string_module: crate::StringModule,
/// }
///
/// #[lockjaw::component(modules = "crate::MyComponentModules")]
/// pub trait MyComponent {
///     fn string(&self) -> String;
/// }
///
/// pub fn main() {
///     let component = MyComponent::build(
///         MyComponentModules{
///             string_module: StringModule { string: "foo".to_owned()}
///         }
///     );
///
///     assert_eq!(component.string(), "foo");  
/// }
/// # test_epilogue!();
/// ```
///
/// If a field is not attributed with `#[builder]`, lockjaw will auto generated it when building the
/// component. The field will be stripped from the manifests.
///
/// ```
/// # #[macro_use] extern crate lockjaw_processor;
/// #[module]
/// pub struct IntModule {
///     // no fields, can be auto generated.
/// }
///
/// #[module_impl]
/// impl IntModule {
///     #[provides]
///     pub fn provide_int(&self) -> i32 {
///         42
///     }
/// }
///
/// #[component_module_manifest]
/// pub struct MyComponentModules {
///     // remove #[builder] for auto generation
///     int_module: crate::IntModule,
/// }
///
/// #[lockjaw::component(modules = "crate::MyComponentModules")]
/// pub trait MyComponent {
///     fn int(&self) -> i32;
/// }
///
/// pub fn main() {
///     let component = MyComponent::build(
///         MyComponentModules{
///             // int_module field stripped
///         }
///     );
///
///     assert_eq!(component.int(), 42);  
/// }
/// # test_epilogue!();
/// ```
///
/// Lockjaw also generates `COMPONENT::new() -> Box<dyn COMPONENT>` if the component does not
/// install any modules, or if no modules have `#[builder]` fields.
///
/// ```
/// # #[macro_use] extern crate lockjaw_processor;
/// #
/// #[component]
/// trait MyComponent {
///     //...
/// }
///
/// pub fn main() {
///     let component: Box<dyn MyComponent> = MyComponent::new();
/// }
/// # test_epilogue!();
/// ```
///
/// Each instance of the component will have independent set of [scoped injections](docs::scoped)
pub use lockjaw_processor::component;
pub use lockjaw_processor::component_module_manifest;
pub use lockjaw_processor::inject;
pub use lockjaw_processor::injectable;
pub use lockjaw_processor::mod_epilogue;
pub use lockjaw_processor::module;
pub use lockjaw_processor::module_impl;
pub use lockjaw_processor::provides;
pub use lockjaw_processor::root_epilogue;
pub use lockjaw_processor::test_epilogue;
pub use lockjaw_processor::test_mod_epilogue;

/// Documentation for concepts that does not belong to individual items.
///
/// No real code are included.
#[cfg(nightly)]
pub mod docs {
    #[doc(include = "../docs/scoped.md")]
    pub mod scoped {}
}

mod doctests;

/// once
#[doc(hidden)]
pub struct Once<T> {
    once: std::sync::Once,
    value: RefCell<MaybeUninit<T>>,
}

#[cfg(nightly)]
#[doc(include = "../README.md")]
mod readme {}

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

/// Wrapper around an injection that may be scoped(owned by the component) or free standing(owned by
/// the item injecting it). Deref to access the content.
///
/// Typically this is used when the dependent does not care who owns the dependency, as it will
/// not try to move it. Injecting scoped dependency as 'T' or injected free standing dependency as
/// '&T' is a compile failure, but both can be injected as 'MaybeScoped<T>'
///
/// # Lifetime
///
/// 'MaybeScoped'\'s lifetime is bounded by the component providing it.
pub enum MaybeScoped<'a, T: ?Sized + 'a> {
    Val(Box<T>),
    Ref(&'a T),
}

impl<T: ?Sized> Deref for MaybeScoped<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            MaybeScoped::Val(val) => val.deref(),
            MaybeScoped::Ref(r) => r,
        }
    }
}

pub fn build_script() {
    // Do nothing. just forcing env var OUT_DIR to be set.
}

#[test]
fn compile_test() {
    let t = trybuild::TestCases::new();
    t.pass("tests/run-pass/*.rs");
    t.compile_fail("tests/compile-fail/*.rs");
}
