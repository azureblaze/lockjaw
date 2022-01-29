Annotates a trait that composes the dependency graph and provides items in the graph (An "injector")
.

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
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    let foo : Foo = component.foo();
}
epilogue!();
```

# Component methods

Methods on the component trait serves as entry points to the component. They can be used to retrieve
bindings inside the component from the outside.

Component methods must take only `&self` as parameter, and return a type that has bindings in the
component. Lockjaw will generate the implementation that returns the binding.

# Component builder

For a trait `Foo` annotated with `#[component]`, a builder method is generated:

```ignore
impl Foo {
    pub fn build(modules: BUILDER_MODULES) -> Box<dyn Foo>
}
```

which an instance of the component, with modules in `modules` installed.
`BUILDER_MODULES` is the [annotated struct](builder_modules) in the
[`builder_modules` metadata](#modules).

If the `builder_modules` metadata is not provided, the `modules` parameter will be omitted, and the
signature becomes `pub fn build() -> Box<dyn Foo>`

# Metadata

Components accept additional metadata in the form of `#[component(key=value, key2=value2)]`.

## `modules`

**Optional** path or array of path to
[`modules`](module) to be installed as fields. Bindings in listed modules will be incorporated into
the dependency graph.

These modules must contain no field. Modules with fields must be provided with
[builder_modules](#builder_modules) instead.

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

## `builder_modules`

**Optional** path or array of path to a struct annotated by
[`#[builder_modules]`](builder_modules), which contains
[`modules`](module) to be installed as fields. Bindings in listed modules will be incorporated into
the dependency graph.

If a module does not contain any field, it can be listed in [`modules`](#modules) instead.

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

#[builder_modules]
pub struct MyBuilderModules {
    module : crate::StringModule,
}
#[component(builder_modules : crate::MyBuilderModules)]
pub trait MyComponent {
    fn string(&self) -> String;
}

fn main() {
    let component = <dyn MyComponent>::build(MyBuilderModules{
        module: StringModule{
            string: "foo".to_owned()
        }
    });
    
    assert_eq!("foo", component.string());
}
epilogue!();
```

# Method attributes

Methods in a component can have additional attributes that affects their behavior.

* [`#[qualified]`](component_attributes::qualified)

Method attributes are nested under `#[component]`, and all nested attributes should be unqualified (
always used as `#[attribute]` instead of `#[lockjaw::attribute]`).