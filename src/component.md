Annotates a trait that composes the dependency graph and provides items in
the graph (An "injector").

```
# #[macro_use] extern crate lockjaw_processor;
#
# lockjaw::prologue!("src/lib.rs");
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
    let component = MyComponentBuilder{}.build();
    let foo : Foo = component.foo();
}
epilogue!();
```

# Generated structs

# `struct [COMPONENT_NAME]Builder`

A struct that can create an instance of the component.

The struct will only be generated if the [`component_builder` metadata](#component_builder) is not
assigned. Otherwise, the struct designated by `component_builder` should be used to create the
component.

See also [`#[component_builder]`](component_builder).

# Metadata

Components accept additional metadata in the form of `#[component(key=value, key2=value2)]`.

## `modules`

**Optional** path or array of path to
[`modules`](module) to be installed as fields. Bindings in listed modules will be incorporated into
the dependency graph.

These modules must contain no field. Modules with fields must be provided with
[component_builder](#component_builder) instead.

```
# #[macro_use] extern crate lockjaw_processor;
# lockjaw::prologue!("src/lib.rs");
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

#[component(modules : [StringModule, UnsignedModule])]
trait MyComponent {
    fn string(&self) -> String;
    fn unsigned(&self) -> u32;
}

# fn main() {}
# epilogue!();
```

## `component_builder`

**Optional** path or array of path to a struct annotated by
[`#[component_builder]`](component_builder), which contains
[`modules`](module) to be installed as fields. Bindings in listed modules will be incorporated into
the dependency graph.

If a module does not contain any fields, it can be listed in [`modules`](#modules) instead.

```
# #[macro_use] extern crate lockjaw_processor;
# lockjaw::prologue!("src/lib.rs");
struct StringModule {
    string : String
}
#[module]
impl StringModule {
    #[provides]
    pub fn provide_string(&self) -> String {
        self.string.clone()
    }
}

#[component_builder]
struct MyComponentBuilder {
    module : crate::StringModule,
}
#[component(component_builder : crate::MyComponentBuilder)]
trait MyComponent {
    fn string(&self) -> String;
}

fn main() {
    let component = MyComponentBuilder {
        module: StringModule{
            string: "foo".to_owned()
        }
    }.build();
    
    assert_eq!("foo", component.string());
}
epilogue!();
```
