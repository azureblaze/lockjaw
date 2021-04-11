Annotates a struct that lists [`modules`](module) to be installed in a
[`component`](component).

The annotated struct will become the parameter for
[`COMPONENT.build()`](component#pub-fn-buildmodules-component_module_manifest---impl-component)

`Modules` not annotated with [`#[builder]`](#builder) are stripped from the struct, as
lockjaw will auto generate them

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

# Field annotations

## `#[builder]`
Annotates a module field that cannot be auto generated (as it is not an empty struct) and must
be explicitly provided to
[`COMPONENT.build()`](component#pub-fn-buildmodules-component_module_manifest---impl-component)

```
# #[macro_use] extern crate lockjaw_processor;
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

#[component_module_manifest]
struct MyModuleManifest {
    #[builder]
    module : crate::StringModule,
}
#[component(modules = "crate::MyModuleManifest")]
trait MyComponent {
    fn string(&self) -> String;
}

fn main() {
    let component = MyComponent::build(MyModuleManifest{
        module: StringModule{
            string: "foo".to_owned()
        }
    });
    
    assert_eq!("foo", component.string());
}
epilogue!();
```