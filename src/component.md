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
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    let foo : Foo = component.foo();
}
epilogue!();
```
# Generated methods

# `pub fn build(modules: BUILDER_MODULES) -> impl COMPONENT`

Create an instance of the component, with modules in `modules` installed.
`BUILDER_MODULES` is the [annotated struct](builder_modules) in the
[`builder_modules` metadata](#modules).

# `pub fn new() -> impl COMPONENT`

Create an instance of the component. Only generated if `builder_modules` is not used.

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
struct MyBuilderModules {
    module : crate::StringModule,
}
#[component(builder_modules : crate::MyBuilderModules)]
trait MyComponent {
    fn string(&self) -> String;
}

fn main() {
    let component = MyComponent::build(MyBuilderModules{
        module: StringModule{
            string: "foo".to_owned()
        }
    });
    
    assert_eq!("foo", component.string());
}
epilogue!();
```
