Annotates a impl block that defines the bindings.

To incorporate a module to the dependency graph, it should be included as a field in the
`modules` field the [component](component) annotation.

```
# use lockjaw::{epilogue, injectable,  component};
# lockjaw::prologue!("src/lib.rs");
use lockjaw::{module};
pub struct FooModule {}

#[module]
impl FooModule {
    #[provides]
    pub fn provide_string() -> String {
        "foo".to_owned()
    }
}

#[component(modules : [FooModule])]
pub trait MyComponent {
    fn string(&self) -> String;
}

pub fn main() {
    let component = <dyn MyComponent>::new();
    assert_eq!(component.string(), "foo");
}
epilogue!();
```

If the module struct contains fields, it must use [`builder_modules`](builder_modules) instead.

```
# use lockjaw::*;
# lockjaw::prologue!("src/lib.rs");
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

#[builder_modules]
pub struct MyBuilderModules {
    foo : FooModule,
}

#[component(builder_modules : MyBuilderModules)]
pub trait MyComponent {
    fn string(&self) -> String;
}

pub fn main() {
    let component = MyComponent::build(MyBuilderModules {
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
# lockjaw::prologue!("src/lib.rs");
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

#[builder_modules]
pub struct MyBuilderModules {
    foo : crate::FooModule,
}

#[component(builder_modules : crate::MyBuilderModules)]
pub trait MyComponent {
    fn string(&self) -> String;
}

pub fn main() {
    let component = MyComponent::build(MyBuilderModules {
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

`#[provides]` accept additional metadata in the form of
`#[provides(key=value, key2=value)]`.

#### scope

**Optional** fully qualified path to a [`component`](component), which makes the returned object a
scoped singleton under the `component`.

The return object will only be provided in the `component`, and all objects generated from the
same `component` instance will share the same scoped returned object. Since it is shared, the scoped
returned object can only be depended on as  `&T` or [`Cl<T>`](ComponentLifetime), and the scoped
returned object or any objects that depends on it will share the lifetime of the
`component`.

```
# use lockjaw::*;
# lockjaw::prologue!("src/lib.rs");

pub struct Foo {}

pub struct FooModule {}

#[module]
impl FooModule {
    #[provides(scope : crate::MyComponent)]
    pub fn provide_foo() -> crate::Foo {
        Foo{}
    }
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

#[component(modules : FooModule)]
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

Annotates a method that binds an implementation to a trait. Whenever the trait is depended on, this
implementation will be provided.

Must take the implementation as the one and only one parameter, and return
[`Cl<dyn T>`](#ComponentLifetime).

The method implementation must be empty. Lockjaw will generate the actual implementation.

The trait can only be depended on as `Cl<'_, dyn T>`, as there are no guarantee whether an
implementation will depend on something that is scoped or not.

Cannot annotate a method that is already annotated with [`#[provides]`](#provides)

```
# use lockjaw::*;
# lockjaw::prologue!("src/lib.rs");
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
    pub fn bind_my_trait(_impl: crate::MyTraitImpl) -> Cl<dyn crate::MyTrait> {}
}

#[component(modules : MyModule)]
pub trait MyComponent {
    fn my_trait(&'_ self) -> Cl<'_, dyn crate::MyTrait>;
}

pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    assert_eq!(component.my_trait().hello(), "hello");
}
epilogue!();
```
### Metadata

`#[binds]` accept additional metadata in the form of
`#[binds(key=value, key2=value)]`.

#### scope

**Optional** fully qualified path to a [`component`](component), which makes the returned trait a
scoped singleton under the `component`.

The return trait will only be provided in the `component`, and all objects generated from the
same `component` instance will share the same scoped returned trait. Since it is shared, the scoped
returned trait can only be depended on as  [`Cl<T>`](ComponentLifetime), and the scoped returned
trait or any objects that depends on it will share the lifetime of the
`component`.

```
# use lockjaw::*;
# use std::ops::Deref;
# lockjaw::prologue!("src/lib.rs");
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
    #[binds(scope : crate::MyComponent)]
    pub fn binds_foo(_impl: crate::FooImpl) -> Cl<dyn crate::Foo> {}
}

pub struct Bar<'a>{
    foo : Cl<'a, dyn crate::Foo>
}
#[injectable]
impl Bar<'_> {
    #[inject]
    pub fn new(foo : Cl<'_, dyn crate::Foo>) -> Bar<'_> {
        Bar { foo }
    }
}

#[component(modules : FooModule)]
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

Module additional metadata in the form of
`#[module(key=value, key2=value)]`.
