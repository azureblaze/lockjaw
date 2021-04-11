Annotates a impl block that defines the bindings.

To incorporate a module to the dependency graph, it should be included as a field in a
[`#[component_module_manifest]`](component_module_manifest), and added to the compoenet.

```
# use lockjaw::{epilogue, injectable, component_module_manifest, component};
use lockjaw::{module};
pub struct FooModule {}

#[module]
impl FooModule {
    #[provides]
    pub fn provide_string() -> String {
        "foo".to_owned()
    }
}

#[component_module_manifest]
pub struct MyModuleManifest {
    foo : crate::FooModule,
}

#[component(modules = "crate::MyModuleManifest")]
pub trait MyComponent {
    fn string(&self) -> String;
}

pub fn main() {
    let component = <dyn MyComponent>::new();
    assert_eq!(component.string(), "foo");
}
epilogue!();
```

If the module struct contains fields, it must be marked as
[`#[builder]`](component_module_manifest#buiilder) in the `#[component_module_manifest]`, and
provided to `COMPONENT.build()`

```
# use lockjaw::*;
pub struct FooModule {
    value : String
}

#[module]
impl FooModule {
    #[provides]
    pub fn provide_string(&self) -> String {
        self.value.clone()
    }
}

#[component_module_manifest]
pub struct MyModuleManifest {
    #[builder]
    foo : crate::FooModule,
}

#[component(modules = "crate::MyModuleManifest")]
pub trait MyComponent {
    fn string(&self) -> String;
}

pub fn main() {
    let component = MyComponent::build(MyModuleManifest {
        foo : FooModule {
            value:"bar".to_owned()
        }
    });
    assert_eq!(component.string(), "bar");
}
epilogue!();
```

# Method annotations

## `#[provides]`

Annotates a method that provides an object into the dependency graph. When an object of the
return type is depended on, this method will be called to create the object. Other dependencies
can be requested with the method parameter. `&self` can also be used to access runtime values
stored in the module.

The return type and parameters (except `&self`) must be fully qualified.

```
# use lockjaw::*;
pub struct Bar {}
#[injectable]
impl Bar {
    #[inject]
    pub fn new()-> Self {
        Self {}
    }
}

impl Bar {
    pub fn get_string(&self) -> String {
        "bar".to_owned()
    }
}

pub struct FooModule {
    value : String
}

#[module]
impl FooModule {
    #[provides]
    pub fn provide_string(&self, bar : crate::Bar) -> String {
        format!("{} {}",self.value.clone(), bar.get_string() )
    }
}

#[component_module_manifest]
pub struct MyModuleManifest {
    #[builder]
    foo : crate::FooModule,
}

#[component(modules = "crate::MyModuleManifest")]
pub trait MyComponent {
    fn string(&self) -> String;
}

pub fn main() {
    let component = MyComponent::build(MyModuleManifest {
        foo : FooModule {
            value:"foo".to_owned()
        }
    });
    assert_eq!(component.string(), "foo bar");
}
epilogue!();

```

Cannot annotate a method that is already annotated with [`#[binds]`](#binds)

### Metadata

`#[provides]` accept addtional metadata in the form of
`#[provides(key="value", key2="value2")]`. Currently all values are string literals.

#### scope

**Optional** fully qualified path to a [`component`](component), which makes the returned object
a scoped singleton under the `component`.

The return object will only be provided in the `component`, and all objects generated from the
same `component` instance will share the same scoped returned object. Since it is shared,
the scoped returned object can only be depended on as  `&T` or [`ComponentLifetime<T>`](ComponentLifetime),
and the scoped returned object or any objects that depends on it will share the lifetime of the
`component`.

```
# use lockjaw::*;

pub struct Foo {}

pub struct FooModule {}

#[module]
impl FooModule {
    #[provides(scope="crate::MyComponent")]
    pub fn provide_foo() -> crate::Foo {
        Foo{}
    }
}

#[component_module_manifest]
pub struct MyModuleManifest {
    foo : crate::FooModule,
}

pub struct Bar<'a>{
    foo : &'a crate::Foo
}

#[injectable]
impl Bar<'_> {
    #[inject]
    pub fn new(foo : &'_ crate::Foo) -> Bar<'_> {
        Bar { foo }
    }
}

#[component(modules = "crate::MyModuleManifest")]
pub trait MyComponent {
    fn bar(&self) -> crate::Bar;
}

pub fn main() {
    let component = <dyn MyComponent>::new();
    let bar1 = component.bar();
    let bar2 = component.bar();
    let bar1_ptr: *const Bar = &bar1;
    let bar2_ptr: *const Bar = &bar2;
    assert_ne!(bar1_ptr, bar2_ptr);
    let foo1_ptr : *const Foo = bar1.foo;
    let foo2_ptr : *const Foo = bar2.foo;
    assert_eq!(foo1_ptr, foo2_ptr);
}
epilogue!();
```

Scoped returned objects are shared and cannot be mutable while they commonly needs mutability.
users must implement internal mutability.

## `#[binds]`

Annotates a method that binds an implementation to a trait. Whenever the trait is depended on,
this implementation will be provided.

Must take the implementation as the one and only one parameter, and return
[`ComponentLifetime<dyn T>`](#ComponentLifetime).

The method implementation must be empty. Lockjaw will generate the actual implementation.

The trait can only be depended on as `ComponentLifetime<'_, dyn T>`, as there are no guaratee whether
an implementation will depend on something that is scoped or not.

Cannot annotate a method that is already annotated with [`#[provides]`](#provides)

```
# use lockjaw::*;
pub trait MyTrait {
    fn hello(&self) -> String;
}

pub struct MyTraitImpl {}

#[injectable]
impl MyTraitImpl {
    #[inject]
    pub fn new() -> Self {
        Self {}
    }
}

impl MyTrait for MyTraitImpl {
    fn hello(&self) -> String {
        "hello".to_owned()
    }
}

pub struct MyModule {}
#[module]
impl MyModule {
    #[binds]
    pub fn bind_my_trait(_impl: crate::MyTraitImpl) -> ComponentLifetime<dyn crate::MyTrait> {}
}

#[component_module_manifest]
pub struct MyModuleManifest {
    my_module: crate::MyModule,
}

#[component(modules = "crate::MyModuleManifest")]
pub trait MyComponent {
    fn my_trait(&'_ self) -> ComponentLifetime<'_, dyn crate::MyTrait>;
}

pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    assert_eq!(component.my_trait().hello(), "hello");
}
epilogue!();
```
### Metadata

`#[binds]` accept addtional metadata in the form of
`#[binds(key="value", key2="value2")]`. Currently all values are string literals.

#### scope

**Optional** fully qualified path to a [`component`](component), which makes the returned trait
a scoped singleton under the `component`.

The return trait will only be provided in the `component`, and all objects generated from the
same `component` instance will share the same scoped returned trait. Since it is shared,
the scoped returned trait can only be depended on as  [`ComponentLifetime<T>`](ComponentLifetime),
and the scoped returned trait or any objects that depends on it will share the lifetime of the
`component`.

```
# use lockjaw::*;
# use std::ops::Deref;
pub trait Foo {}

pub struct FooImpl{}
#[injectable]
impl FooImpl {
    #[inject]
    pub fn new() -> Self {
        Self {}
    }
}

impl Foo for FooImpl {}

pub struct FooModule {}

#[module]
impl FooModule {
    #[binds(scope="crate::MyComponent")]
    pub fn binds_foo(_impl: crate::FooImpl) -> ComponentLifetime<dyn crate::Foo> {}
}

#[component_module_manifest]
pub struct MyModuleManifest {
    foo : crate::FooModule,
}

pub struct Bar<'a>{
    foo : ComponentLifetime<'a, dyn crate::Foo>
}
#[injectable]
impl Bar<'_> {
    #[inject]
    pub fn new(foo : ComponentLifetime<'_, dyn crate::Foo>) -> Bar<'_> {
        Bar { foo }
    }
}

#[component(modules = "crate::MyModuleManifest")]
pub trait MyComponent {
    fn bar(&self) -> crate::Bar;
}

pub fn main() {
    let component = <dyn MyComponent>::new();
    let bar1 = component.bar();
    let bar2 = component.bar();
    let bar1_ptr: *const Bar = &bar1;
    let bar2_ptr: *const Bar = &bar2;
    assert_ne!(bar1_ptr, bar2_ptr);
    let foo1_ptr : *const dyn Foo = bar1.foo.deref();
    let foo2_ptr : *const dyn Foo = bar2.foo.deref();
    assert_eq!(foo1_ptr, foo2_ptr);
}
epilogue!();
```

Scoped returned objects are shared and cannot be mutable while they commonly needs mutability.
users must implement internal mutability.

# Metadata

Module accept addtional metadata in the form of
`#[module(key="value", key2="value2")]`. Currently all values are string literals.

## `path`
**Optional** [path](https://doc.rust-lang.org/reference/paths.html) relative to the path of the
current file.

Lockjaw retrieves the path of the current file from [`epilogue!()`](epilogue) and
[`mod_epilogue!()`](mod_epilogue), but if the `module` is nested under a
[`mod`](https://doc.rust-lang.org/reference/items/modules.html) then the extra path must be
specified.

```
# use lockjaw::{epilogue, injectable, component_module_manifest, component};
mod nested {
    use lockjaw::module;
    pub struct FooModule {}

    #[module(path = "nested")]
    impl FooModule {
        #[provides]
        pub fn provide_string() -> String {
            "foo".to_owned()
        }
    }
}

#[component_module_manifest]
pub struct MyModuleManifest {
    foo : crate::nested::FooModule,
}

#[component(modules = "crate::MyModuleManifest")]
pub trait MyComponent {
    fn string(&self) -> String;
}

pub fn main() {
    let component = <dyn MyComponent>::new();
    assert_eq!(component.string(), "foo");
}
epilogue!();
```