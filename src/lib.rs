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

/// Annotates a trait that composes the dependency graph and provides items in
/// the graph (An "injector").
///
/// ```
/// # #[macro_use] extern crate lockjaw_processor;
/// #
/// # struct Foo{}
/// #
/// # #[injectable]
/// # impl Foo {
/// #     #[inject]
/// #     pub fn new() -> Self {
/// #         Self {}
/// #     }
/// # }
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
/// epilogue!();
/// ```
/// # Generated methods
///
/// # `pub fn build(modules: COMPONENT_MODULE_MANIFEST) -> impl COMPONENT`
///
/// Create an instance of the component, with modules in `modules` installed.
/// `COMPONENT_MODULE_MANIFEST` is the [annotated struct](component_module_manifest) in the
/// [`modules` metadata](#modules).
///
/// NOTE: fields not annotated with [`#[builder]`](component_module_manifest#builder) will be
/// stripped from the struct and should not be specified as they are auto-generated.
///
/// # `pub fn new() -> impl COMPONENT`
///
/// Create an instance of the component. Only generated if no module instances are required,
/// which means either the component does not install any module with the [`modules`](#modules)
/// metadata, or none of the fields in
/// [`#[component_module_manifest]`](component_module_manifest) struct are annotated with
/// [`#[builder]`](component_module_manifest#builder).
///
/// # Metadata
///
/// Components accept addtional metadata in the form of
/// `#[component(key="value", key2="value2")]`. Currently all values are string literals.
///
/// ## `modules`
///
/// **Optional** comma-separated, fully qualifed path a struct annotated by
/// [`#[component_module_manifest]`](component_module_manifest), which contains
/// [`modules`](module) to be installed as fields. Bindings in listed modules will be
/// incorporated into the dependency graph.
///
/// ```
/// # #[macro_use] extern crate lockjaw_processor;
/// # struct StringModule {}
/// # #[module]
/// # impl StringModule {
/// #     #[provides]
/// #     pub fn provide_string() -> String {
/// #         "string".to_owned()
/// #     }
/// # }
/// #
/// # struct UnsignedModule {}
/// # #[module]
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
/// # epilogue!();
/// ```
///
/// ## `path`
/// **Optional** [path](https://doc.rust-lang.org/reference/paths.html) relative to the path of the
/// current file.
///
/// Lockjaw retrieves the path of the current file from [`epilogue!()`](epilogue) and
/// [`mod_epilogue!()`](mod_epilogue), but if the `component` is nested under a
/// [`mod`](https://doc.rust-lang.org/reference/items/modules.html) then the extra path must be
/// specified.
///
/// ```
/// # use lockjaw::{epilogue, injectable};
/// # pub struct Foo {}
/// #
/// # #[injectable]
/// # impl Foo {
/// #     #[inject]
/// #     pub fn new() -> Self {
/// #         Self {}
/// #     }
/// # }
///
/// mod nested {
///     #[lockjaw::component(path = "nested")]
///     pub trait MyComponent {
///         fn foo(&self) -> crate::Foo;
///     }
/// }
/// pub fn main() {
///     let component = nested::MyComponent::new();
///     component.foo();
/// }
/// epilogue!();
/// ```
pub use lockjaw_processor::component;

/// Annotates a struct that lists [`modules`](module) to be installed in a
/// [`component`](component).
///
/// The annotated struct will become the parameter for
/// [`COMPONENT.build()`](component#pub-fn-buildmodules-component_module_manifest---impl-component)
///
/// `Modules` not annotated with [`#[builder]`](#builder) are stripped from the struct, as
/// lockjaw will auto generate them
///
/// ```
/// # #[macro_use] extern crate lockjaw_processor;
/// # struct StringModule {}
/// # #[module]
/// # impl StringModule {
/// #     #[provides]
/// #     pub fn provide_string() -> String {
/// #         "string".to_owned()
/// #     }
/// # }
/// #
/// # struct UnsignedModule {}
/// # #[module]
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
/// # epilogue!();
/// ```
///
/// # Field annotations
///
/// ## `#[builder]`
/// Annotates a module field that cannot be auto generated (as it is not an empty struct) and must
/// be explicitly provided to
/// [`COMPONENT.build()`](component#pub-fn-buildmodules-component_module_manifest---impl-component)
///
/// ```
/// # #[macro_use] extern crate lockjaw_processor;
/// struct StringModule {
///     string : String
/// }
/// #[module]
/// impl StringModule {
///     #[provides]
///     pub fn provide_string(&self) -> String {
///         self.string.clone()
///     }
/// }
///
/// #[component_module_manifest]
/// struct MyModuleManifest {
///     #[builder]
///     module : crate::StringModule,
/// }
/// #[component(modules = "crate::MyModuleManifest")]
/// trait MyComponent {
///     fn string(&self) -> String;
/// }
///
/// fn main() {
///     let component = MyComponent::build(MyModuleManifest{
///         module: StringModule{
///             string: "foo".to_owned()
///         }
///     });
///     
///     assert_eq!("foo", component.string());
/// }
/// epilogue!();
/// ```
pub use lockjaw_processor::component_module_manifest;

/// Resolves the dependency graph and generate component code. Must be called in in the crate root
/// (`lib.rs` or `main.rs`), after any other lockjaw macros, and outside any `mod`/functions
///
/// a unit test will be generated to ensure it is called in the correct file.
///
/// # Parameters
/// The macro accepts additional parameters in the from of identifiers. Regular users should rarely
/// need to use these.
///
/// ## `debug_output`
/// Writes the `epilogue!()` output to a file and `include!()` it, instead of inserting a hygienic
/// token stream. This allows easier debugging of code generation issues.
pub use lockjaw_processor::epilogue;

/// Annotates a struct impl that can be provided to the dependency graph.
///
/// ```
/// # use lockjaw::{epilogue, injectable};
/// # #[macro_use] extern crate lockjaw_processor;
/// struct Bar{}
///
/// #[injectable]
/// impl Bar {
///     #[inject]
///     pub fn new() -> Self {
///         Self {}
///     }
/// }
///
/// struct Foo{
///     bar : crate::Bar,
///     s : String,
/// }
///
/// #[injectable]
/// impl Foo {
///     #[inject]
///     pub fn new(bar : crate::Bar,) -> Self {
///         Self {bar, s: "foo".to_owned()}
///     }
/// }
///
/// #[component]
/// trait MyComponent {
///     fn foo(&self) -> crate::Foo;
/// }
///
/// pub fn main() {
///     let component = MyComponent::new();
///     let foo = component.foo();
/// }
/// epilogue!();
/// ```
///
/// # Methods
///
/// ## `#[inject]` methods
/// Denotes the method as "injection constructor", which is the method lockjaw will call to create
/// the object.
///
/// One and only one method must be annotated with `#[inject]` in an `#[injectable]` struct. The
/// method must be static, and must return an instance of the struct.
///
/// The method can request other injectable objects with its parameters. Lockjaw will fulfil those
/// objects before calling the injection constructor.
///
/// # Metadata
///
/// Injectables accept addtional metadata in the form of
/// `#[injectable(key="value", key2="value2")]`. Currently all values are string literals.
///
/// ## `path`
/// **Optional** [path](https://doc.rust-lang.org/reference/paths.html) relative to the path of the
/// current file.
///
/// Lockjaw retrieves the path of the current file from [`epilogue!()`](epilogue) and
/// [`mod_epilogue!()`](mod_epilogue), but if the `injectable` is nested under a
/// [`mod`](https://doc.rust-lang.org/reference/items/modules.html) then the extra path must be
/// specified.
///
/// ```
/// # use lockjaw::{epilogue, injectable};
///
/// mod nested {
///     pub struct Foo {}
///     #[lockjaw::injectable(path = "nested")]
///     impl Foo {
///         #[inject]
///         pub fn new()-> Self {
///             Self {}
///         }
///     }
/// }
/// #[lockjaw::component]
/// pub trait MyComponent {
///     fn foo(&self) -> crate::nested::Foo;
/// }
///
/// pub fn main() {
///     let component = MyComponent::new();
///     component.foo();
/// }
/// epilogue!();
/// ```
/// ## `scope`
///
/// **Optional** fully qualified path to a [`component`](component), which makes the `injectable` a
/// scoped singleton under the `component`.
///
/// The `injectable` will only be provided in the `component`, and all objects generated from the
/// same `component` instance will share the same scoped `injecetable` instance. Since it is shared,
/// the scoped `injectable` can only be depended on as  `&T` or [`MaybeScoped<T>`](MaybeScoped), and
/// the scoped `injectable` or any objects that depends on it will share the lifetime of the
/// `component`.
///
/// ```
/// # use lockjaw::{epilogue, injectable};
/// pub struct Foo {}
///
/// #[injectable(scope = "crate::MyComponent")]
/// impl Foo {
///     #[inject]
///     pub fn new()-> Self {
///         Self {}
///     }
/// }
///
/// pub struct Bar<'a>{
///     foo : &'a crate::Foo
/// }
///
/// #[injectable]
/// impl Bar<'_> {
///     #[inject]
///     pub fn new(foo : &'_ crate::Foo) -> Bar<'_> {
///         Bar {foo}
///     }
/// }
///
/// #[lockjaw::component]
/// pub trait MyComponent {
///     fn bar(&self) -> crate::Bar;
/// }
///
/// pub fn main() {
///     let component = MyComponent::new();
///     let bar1 = component.bar();
///     let bar2 = component.bar();
///     let bar1_ptr: *const Bar = &bar1;
///     let bar2_ptr: *const Bar = &bar2;
///     assert_ne!(bar1_ptr, bar2_ptr);
///     let foo1_ptr : *const Foo = bar1.foo;
///     let foo2_ptr : *const Foo = bar2.foo;
///     assert_eq!(foo1_ptr, foo2_ptr);
/// }
/// epilogue!();
/// ```
///
/// Scoped `injectables` are shared and cannot be mutable while they commonly needs mutability.
/// users must implement internal mutability.
pub use lockjaw_processor::injectable;

/// Must be called at the end of a non-root (not `lib.rs` or `main.rs`) file that uses lockjaw
/// To let it know a file has concluded. The path from the crate root to the current `mod` the file
/// represents must be passed in as a string literal. i.e.
/// * `src/foo.rs` => `mod_epilogue!("foo");`
/// * `src/bar/mod.rs` => `mod_epilogue!("bar");`
/// * `src/bar/baz.rs` => `mod_epilogue!("bar::baz");`
///
/// Lockjaw requires this information to resolve the path of the bindings in the current file.
pub use lockjaw_processor::mod_epilogue;

/// Annotates a impl block that defines the bindings.
///
/// To incorporate a module to the dependency graph, it should be included as a field in a
/// [`#[component_module_manifest]`](component_module_manifest), and added to the compoenet.
///
/// ```
/// # use lockjaw::{epilogue, injectable, component_module_manifest, component};
/// use lockjaw::{module};
/// pub struct FooModule {}
///
/// #[module]
/// impl FooModule {
///     #[provides]
///     pub fn provide_string() -> String {
///         "foo".to_owned()
///     }
/// }
///
/// #[component_module_manifest]
/// pub struct MyModuleManifest {
///     foo : crate::FooModule,
/// }
///
/// #[component(modules = "crate::MyModuleManifest")]
/// pub trait MyComponent {
///     fn string(&self) -> String;
/// }
///
/// pub fn main() {
///     let component = MyComponent::new();
///     assert_eq!(component.string(), "foo");
/// }
/// epilogue!();
/// ```
///
/// If the module struct contains fields, it must be marked as
/// [`#[builder]`](component_module_manifest#buiilder) in the `#[component_module_manifest]`, and
/// provided to `COMPONENT.build()`
///
/// ```
/// # use lockjaw::*;
/// pub struct FooModule {
///     value : String
/// }
///
/// #[module]
/// impl FooModule {
///     #[provides]
///     pub fn provide_string(&self) -> String {
///         self.value.clone()
///     }
/// }
///
/// #[component_module_manifest]
/// pub struct MyModuleManifest {
///     #[builder]
///     foo : crate::FooModule,
/// }
///
/// #[component(modules = "crate::MyModuleManifest")]
/// pub trait MyComponent {
///     fn string(&self) -> String;
/// }
///
/// pub fn main() {
///     let component = MyComponent::build(MyModuleManifest {
///         foo : FooModule {
///             value:"bar".to_owned()
///         }
///     });
///     assert_eq!(component.string(), "bar");
/// }
/// epilogue!();
/// ```
///
/// # Method annotations
///
/// ## `#[provides]`
///
/// Annotates a method that provides an object into the dependency graph. When an object of the
/// return type is depended on, this method will be called to create the object. Other dependencies
/// can be requested with the method parameter. `&self` can also be used to access runtime values
/// stored in the module.
///
/// The return type and parameters (except `&self`) must be fully qualified.
///
/// ```
/// # use lockjaw::*;
/// pub struct Bar {}
/// #[injectable]
/// impl Bar {
///     #[inject]
///     pub fn new()-> Self {
///         Self {}
///     }
/// }
///
/// impl Bar {
///     pub fn get_string(&self) -> String {
///         "bar".to_owned()
///     }
/// }
///
/// pub struct FooModule {
///     value : String
/// }
///
/// #[module]
/// impl FooModule {
///     #[provides]
///     pub fn provide_string(&self, bar : crate::Bar) -> String {
///         format!("{} {}",self.value.clone(), bar.get_string() )
///     }
/// }
///
/// #[component_module_manifest]
/// pub struct MyModuleManifest {
///     #[builder]
///     foo : crate::FooModule,
/// }
///
/// #[component(modules = "crate::MyModuleManifest")]
/// pub trait MyComponent {
///     fn string(&self) -> String;
/// }
///
/// pub fn main() {
///     let component = MyComponent::build(MyModuleManifest {
///         foo : FooModule {
///             value:"foo".to_owned()
///         }
///     });
///     assert_eq!(component.string(), "foo bar");
/// }
/// epilogue!();
///
/// ```
///
/// Cannot annotate a method that is already annotated with [`#[binds]`](#binds)
///
/// ### Metadata
///
/// `#[provides]` accept addtional metadata in the form of
/// `#[provides(key="value", key2="value2")]`. Currently all values are string literals.
///
/// #### scope
///
/// **Optional** fully qualified path to a [`component`](component), which makes the returned object
/// a scoped singleton under the `component`.
///
/// The return object will only be provided in the `component`, and all objects generated from the
/// same `component` instance will share the same scoped returned object. Since it is shared,
/// the scoped returned object can only be depended on as  `&T` or [`MaybeScoped<T>`](MaybeScoped),
/// and the scoped returned object or any objects that depends on it will share the lifetime of the
/// `component`.
///
/// ```
/// # use lockjaw::*;
///
/// pub struct Foo {}
///
/// pub struct FooModule {}
///
/// #[module]
/// impl FooModule {
///     #[provides(scope="crate::MyComponent")]
///     pub fn provide_foo() -> crate::Foo {
///         Foo{}
///     }
/// }
///
/// #[component_module_manifest]
/// pub struct MyModuleManifest {
///     foo : crate::FooModule,
/// }
///
/// pub struct Bar<'a>{
///     foo : &'a crate::Foo
/// }
///
/// #[injectable]
/// impl Bar<'_> {
///     #[inject]
///     pub fn new(foo : &'_ crate::Foo) -> Bar<'_> {
///         Bar { foo }
///     }
/// }
///
/// #[component(modules = "crate::MyModuleManifest")]
/// pub trait MyComponent {
///     fn bar(&self) -> crate::Bar;
/// }
///
/// pub fn main() {
///     let component = MyComponent::new();
///     let bar1 = component.bar();
///     let bar2 = component.bar();
///     let bar1_ptr: *const Bar = &bar1;
///     let bar2_ptr: *const Bar = &bar2;
///     assert_ne!(bar1_ptr, bar2_ptr);
///     let foo1_ptr : *const Foo = bar1.foo;
///     let foo2_ptr : *const Foo = bar2.foo;
///     assert_eq!(foo1_ptr, foo2_ptr);
/// }
/// epilogue!();
/// ```
///
/// Scoped returned objects are shared and cannot be mutable while they commonly needs mutability.
/// users must implement internal mutability.
///
/// ## `#[binds]`
///
/// Annotates a method that binds an implementation to a trait. Whenever the trait is depended on,
/// this implementation will be provided.
///
/// Must take the implementation as the one and only one parameter, and return a trait with the
/// `impl` keyword.
///
/// The method implementation must be empty. Lockjaw will generate the actual implementation.
///
/// The trait can only be depended on as `MaybeScoped<'_, dyn T>`, as there are no guaratee whether
/// an implementation will depend on something that is scoped or not.
///
/// Cannot annotate a method that is already annotated with [`#[provides]`](#provides)
///
/// ```
/// # use lockjaw::*;
/// pub trait MyTrait {
///     fn hello(&self) -> String;
/// }
///
/// pub struct MyTraitImpl {}
///
/// #[injectable]
/// impl MyTraitImpl {
///     #[inject]
///     pub fn new() -> Self {
///         Self {}
///     }
/// }
///
/// impl MyTrait for MyTraitImpl {
///     fn hello(&self) -> String {
///         "hello".to_owned()
///     }
/// }
///
/// pub struct MyModule {}
/// #[module]
/// impl MyModule {
///     #[binds]
///     pub fn bind_my_trait(_impl: crate::MyTraitImpl) -> impl crate::MyTrait {}
/// }
///
/// #[component_module_manifest]
/// pub struct MyModuleManifest {
///     my_module: crate::MyModule,
/// }
///
/// #[component(modules = "crate::MyModuleManifest")]
/// pub trait MyComponent {
///     fn my_trait(&'_ self) -> MaybeScoped<'_, dyn crate::MyTrait>;
/// }
///
/// pub fn main() {
///     let component: Box<dyn MyComponent> = MyComponent::new();
///     assert_eq!(component.my_trait().hello(), "hello");
/// }
/// epilogue!();
/// ```
/// ### Metadata
///
/// `#[binds]` accept addtional metadata in the form of
/// `#[binds(key="value", key2="value2")]`. Currently all values are string literals.
///
/// #### scope
///
/// **Optional** fully qualified path to a [`component`](component), which makes the returned trait
/// a scoped singleton under the `component`.
///
/// The return trait will only be provided in the `component`, and all objects generated from the
/// same `component` instance will share the same scoped returned trait. Since it is shared,
/// the scoped returned trait can only be depended on as  [`MaybeScoped<T>`](MaybeScoped),
/// and the scoped returned trait or any objects that depends on it will share the lifetime of the
/// `component`.
///
/// ```
/// # use lockjaw::*;
/// # use std::ops::Deref;
/// pub trait Foo {}
///
/// pub struct FooImpl{}
/// #[injectable]
/// impl FooImpl {
///     #[inject]
///     pub fn new() -> Self {
///         Self {}
///     }
/// }
///
/// impl Foo for FooImpl {}
///
/// pub struct FooModule {}
///
/// #[module]
/// impl FooModule {
///     #[binds(scope="crate::MyComponent")]
///     pub fn binds_foo(_impl: crate::FooImpl) -> impl crate::Foo {}
/// }
///
/// #[component_module_manifest]
/// pub struct MyModuleManifest {
///     foo : crate::FooModule,
/// }
///
/// pub struct Bar<'a>{
///     foo : MaybeScoped<'a, dyn crate::Foo>
/// }
/// #[injectable]
/// impl Bar<'_> {
///     #[inject]
///     pub fn new(foo : MaybeScoped<'_, dyn crate::Foo>) -> Bar<'_> {
///         Bar { foo }
///     }
/// }
///
/// #[component(modules = "crate::MyModuleManifest")]
/// pub trait MyComponent {
///     fn bar(&self) -> crate::Bar;
/// }
///
/// pub fn main() {
///     let component = MyComponent::new();
///     let bar1 = component.bar();
///     let bar2 = component.bar();
///     let bar1_ptr: *const Bar = &bar1;
///     let bar2_ptr: *const Bar = &bar2;
///     assert_ne!(bar1_ptr, bar2_ptr);
///     let foo1_ptr : *const dyn Foo = bar1.foo.deref();
///     let foo2_ptr : *const dyn Foo = bar2.foo.deref();
///     assert_eq!(foo1_ptr, foo2_ptr);
/// }
/// epilogue!();
/// ```
///
/// Scoped returned objects are shared and cannot be mutable while they commonly needs mutability.
/// users must implement internal mutability.
///
/// # Metadata
///
/// Module accept addtional metadata in the form of
/// `#[module(key="value", key2="value2")]`. Currently all values are string literals.
///
/// ## `path`
/// **Optional** [path](https://doc.rust-lang.org/reference/paths.html) relative to the path of the
/// current file.
///
/// Lockjaw retrieves the path of the current file from [`epilogue!()`](epilogue) and
/// [`mod_epilogue!()`](mod_epilogue), but if the `module` is nested under a
/// [`mod`](https://doc.rust-lang.org/reference/items/modules.html) then the extra path must be
/// specified.
///
/// ```
/// # use lockjaw::{epilogue, injectable, component_module_manifest, component};
/// mod nested {
///     use lockjaw::module;
///     pub struct FooModule {}
///
///     #[module(path = "nested")]
///     impl FooModule {
///         #[provides]
///         pub fn provide_string() -> String {
///             "foo".to_owned()
///         }
///     }
/// }
///
/// #[component_module_manifest]
/// pub struct MyModuleManifest {
///     foo : crate::nested::FooModule,
/// }
///
/// #[component(modules = "crate::MyModuleManifest")]
/// pub trait MyComponent {
///     fn string(&self) -> String;
/// }
///
/// pub fn main() {
///     let component = MyComponent::new();
///     assert_eq!(component.string(), "foo");
/// }
/// epilogue!();
/// ```
pub use lockjaw_processor::module;
#[doc(hidden)]
pub use lockjaw_processor::private_root_epilogue;
#[doc(hidden)]
pub use lockjaw_processor::private_test_epilogue;
#[doc(hidden)]
pub use lockjaw_processor::test_mod_epilogue;

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

/// Function that must be called inside the
/// [cargo build script](https://doc.rust-lang.org/cargo/reference/build-scripts.html) to setup the
/// lockjaw environment.
///
/// lockjaw should be added to `[build-dependencies]` of the crate.
///
/// ```
/// // build.rs
/// fn main() {
///     lockjaw::build_script();
/// }
/// ```
pub fn build_script() {
    // Do nothing. just forcing env var OUT_DIR to be set.
}
