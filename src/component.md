Annotates a trait that composes the dependency graph and provides items in
the graph (An "injector").

```
# #[macro_use] extern crate lockjaw_processor;
#
# struct Foo{}
#
# #[injectable]
# impl Foo {
#     #[inject]
#     pub fn new() -> Self {
#         Self {}
#     }
# }
#
#[component]
trait MyComponent {
    fn foo(&self) -> crate::Foo;
}

pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    let foo : Foo = component.foo();
}
epilogue!();
```
# Generated methods

# `pub fn build(modules: COMPONENT_MODULE_MANIFEST) -> impl COMPONENT`

Create an instance of the component, with modules in `modules` installed.
`COMPONENT_MODULE_MANIFEST` is the [annotated struct](component_module_manifest) in the
[`modules` metadata](#modules).

NOTE: fields not annotated with [`#[builder]`](component_module_manifest#builder) will be
stripped from the struct and should not be specified as they are auto-generated.

# `pub fn new() -> impl COMPONENT`

Create an instance of the component. Only generated if no module instances are required,
which means either the component does not install any module with the [`modules`](#modules)
metadata, or none of the fields in
[`#[component_module_manifest]`](component_module_manifest) struct are annotated with
[`#[builder]`](component_module_manifest#builder).

# Metadata

Components accept addtional metadata in the form of
`#[component(key="value", key2="value2")]`. Currently all values are string literals.

## `modules`

**Optional** comma-separated, fully qualifed path a struct annotated by
[`#[component_module_manifest]`](component_module_manifest), which contains
[`modules`](module) to be installed as fields. Bindings in listed modules will be
incorporated into the dependency graph.

```
# #[macro_use] extern crate lockjaw_processor;
# struct StringModule {}
# #[module]
# impl StringModule {
#     #[provides]
#     pub fn provide_string() -> String {
#         "string".to_owned()
#     }
# }
#
# struct UnsignedModule {}
# #[module]
# impl UnsignedModule {
#     #[provides]
#     pub fn provide_unsigned() -> u32 {
#         42
#     }
# }
#

#[component_module_manifest]
struct MyModuleManifest {
    string : crate::StringModule,
    unsigned : crate::UnsignedModule
}
#[component(modules = "crate::MyModuleManifest")]
trait MyComponent {
    fn string(&self) -> String;
    fn unsigned(&self) -> u32;
}

# fn main() {}
# epilogue!();
```

## `path`
**Optional** [path](https://doc.rust-lang.org/reference/paths.html) relative to the path of the
current file.

Lockjaw retrieves the path of the current file from [`epilogue!()`](epilogue) and
[`mod_epilogue!()`](mod_epilogue), but if the `component` is nested under a
[`mod`](https://doc.rust-lang.org/reference/items/modules.html) then the extra path must be
specified.

```
# use lockjaw::{epilogue, injectable};
# pub struct Foo {}
#
# #[injectable]
# impl Foo {
#     #[inject]
#     pub fn new() -> Self {
#         Self {}
#     }
# }

mod nested {
    #[lockjaw::component(path = "nested")]
    pub trait MyComponent {
        fn foo(&self) -> crate::Foo;
    }
}
pub fn main() {
    let component = <dyn nested::MyComponent>::new();
    component.foo();
}
epilogue!();
```