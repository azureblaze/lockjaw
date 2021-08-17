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

# Method attributes

Methods in a module must have one of `#[provides]`/`#[binds]`/`#[binds_option_of]`/`#[multibinds]`
to denote the binding type. It may also have additional attributes that affects the behavior of the
binding.

## binding types

### `#[provides]`

Annotates a method that provides an object into the dependency graph. When an object of the return
type is depended on, this method will be called to create the object. Other dependencies can be
requested with the method parameter. `&self` can also be used to access runtime values stored in the
module.

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

#### Metadata

`#[provides]` accept additional metadata in the form of
`#[provides(key=value, key2=value)]`.

##### scope

**Optional** fully qualified path to a [`component`](component), which makes the returned object a
scoped singleton under the `component`.

The return object will only be provided in the `component`, and all objects generated from the
same `component` instance will share the same scoped returned object. Since it is shared, the scoped
returned object can only be depended on as  `&T` or [`Cl<T>`](Cl), and the scoped returned object or
any objects that depends on it will share the lifetime _of_ the
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

### `#[binds]`

Annotates a method that binds an implementation to a trait. Whenever the trait is depended on, this
implementation will be provided.

Must take the implementation as the one and only one parameter, and return
[`Cl<dyn T>`](#Cl).

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

### `#[binds_option_of]`

Declares an optional binding. If `#[binds_option_of] pub fn option_foo()->Option<Foo>` is declared,
injecting `Option<Foo>` will result in `Some(Foo)` if `Foo` is bound elsewhere. Otherwise it results
in `None`.

Typically this is used if an optional feature is provided by another module which may not be
included in the component.

### `#[multibinds]`

Declares that a `Vec<T>` or `HashMap<K,V>` is a multibinding. If [#[into_vec]](#into_vec)/
[#[elements_into_vec]](#elements_into_vec)/[#[into_map]](#into_map) exists in the same graph this is
not necessary, but if the collection is empty lockjaw needs to know that it is indeed a multibinding
collection that is currently empty, instead of the user trying to depend on a type that is not
bound.

## Binding modifiers

### `#[into_vec]`

Denotes the return value of the binding should be collected into a `Vec<T>`. `Vec<T>` can then be
depended on to access all bindings of `T`.

A module provide the binding to the `Vec<T>` at most once. However if 2 different module provides a
binding with the same value it will not be deduplicated.

The counterpart of `#[into_vec]` in Dagger is `@IntoSet`. Since `eq`/`hash` is less universally
available in Rust `Vec<T>` is the chosen collection.

```
# use lockjaw::*;
# lockjaw::prologue!("src/lib.rs");
struct MyModule;

#[module]
impl MyModule {

    #[provides]
    #[into_vec]
    pub fn provide_string1() -> String {
        "string1".to_owned()
    }

    #[provides]
    #[into_vec]
    pub fn provide_string2() -> String {
        "string2".to_owned()
    }
}

#[component(modules: MyModule)]
pub trait MyComponent {
    fn vec_string(&self) -> Vec<String>;
}

pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    let v = component.vec_string();
    assert!(v.contains(&"string1".to_owned()));
    assert!(v.contains(&"string2".to_owned()));
}

epilogue!();
```

### `#[elements_into_vec]`

Similar to [`#[into_vec]`](#into_vec) but instead of a single element, all elements in the returned
`Vec<T>` is merged into the `Vec<T>` binding. This allows the module to inject multiple elements
into the `Vec<T>`, or conditionally inject no elements.

```
# use lockjaw::*;
# lockjaw::prologue!("src/lib.rs");
struct MyModule;

#[module]
impl MyModule {

    #[provides]
    #[into_vec]
    pub fn provide_string1() -> String {
        "string1".to_owned()
    }

    #[provides]
    #[elements_into_vec]
    pub fn provide_string2() -> Vec<String> {
        vec!["string2".to_owned(), "string3".to_owned()]
    }
    
    #[provides]
    #[elements_into_vec]
    pub fn provide_string4() -> Vec<String> {
        if true {
            vec![]        
        } else {
            vec!["string4".to_owned()]
        }
    }
}

#[component(modules: MyModule)]
pub trait MyComponent {
    fn vec_string(&self) -> Vec<String>;
}

pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    let v = component.vec_string();
    assert!(v.contains(&"string1".to_owned()));
    assert!(v.contains(&"string2".to_owned()));
    assert!(v.contains(&"string3".to_owned()));
    
    assert!(!v.contains(&"string4".to_owned()));
}

epilogue!();
```

### `#[into_map]`

Denotes the return value of the binding should be collected into a `HashMap<K,V>`. `HashMap<K,V>`
can then be depended on to access all bindings.

The value type of the map is determined by the return type. The key type is determined by additional
metadata on the attribute in the form of `#[into_map(metadata_key: metadata_value)]`.

Keys must be compile time constant.

#### Metadata key `string_key`

The map type is be `HashMap<String, V>`. The metadata should have a string value which will be used
as the key for the binding.

#### Metadata key `i32_key`

The map type is be `HashMap<i32, V>`. The metadata should have a `i32` integer value which will be
used as the key for the binding.

#### Metadata key `enum_key`

The map type is be `HashMap<E, V>` where `E` is the type of the enum. The metadata be a path to the
enum value which wil be used as the key for the binding. The enum must be a simple enum (with no
structs, etc.), and must implement `Eq` and `Hash`

```
# use lockjaw::*;
# use std::collections::HashMap;
# lockjaw::prologue!("src/lib.rs");

#[derive(Eq, PartialEq, Hash)]
pub enum E {
    Foo,
    Bar,
}

pub struct MyModule {}

#[module]
impl MyModule {

    #[provides]
    #[into_map(string_key: "1")]
    pub fn provide_string1() -> String {
        "string1".to_owned()
    }

    #[provides]
    #[into_map(string_key: "2")]
    pub fn provide_string2() -> String {
        "string2".to_owned()
    }

    #[provides]
    #[into_map(i32_key: 1)]
    pub fn provide_i32_string1() -> String {
        "i32_string1".to_owned()
    }

    #[provides]
    #[into_map(i32_key: 2)]
    pub fn provide_i32_string2() -> String {
        "i32_string2".to_owned()
    }

    #[provides]
    #[into_map(enum_key: E::Foo)]
    pub fn provide_enum_string1() -> String {
        "Foo".to_owned()
    }

    #[provides]
    #[into_map(enum_key: E::Bar)]
    pub fn provide_enum_string2() -> String {
        "Bar".to_owned()
    }
}

#[component(modules: [MyModule])]
pub trait MyComponent {
    fn string_map(&self) -> std::collections::HashMap<String, String>;
    fn i32_map(&self) -> std::collections::HashMap<i32, String>;
    fn enum_map(&self) -> std::collections::HashMap<E, String>;
}

pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    
    let string_map = component.string_map();
    assert_eq!(string_map.get("1").unwrap(), "string1");
    assert_eq!(string_map.get("2").unwrap(), "string2");
    
    let i32_map = component.i32_map();
    assert_eq!(i32_map.get(&1).unwrap(), "i32_string1");
    assert_eq!(i32_map.get(&2).unwrap(), "i32_string2");
    
    let enum_map = component.enum_map();
    assert_eq!(enum_map.get(&E::Foo).unwrap(), "Foo");
    assert_eq!(enum_map.get(&E::Bar).unwrap(), "Bar");
}

epilogue!();
```

### `#[qualified]`

Designates a [qualifier](qualifier) to the return type, so they can be seperated bindings of the
same type.

```
# use lockjaw::*;
# lockjaw::prologue!("src/lib.rs");

#[qualifier]
pub struct Foo;

#[qualifier]
pub struct Bar;

pub struct MyModule {}

#[module]
impl MyModule {
    #[provides]
    #[qualified(Foo)]
    pub fn provide_foo_string() -> String {
        "foo".to_owned()
    }
    
    #[provides]
    #[qualified(Bar)]
    pub fn provide_bar_string() -> String {
        "bar".to_owned()
    }
    
    #[provides]
    pub fn provide_regular_string() -> String {
        "regular".to_owned()
    }
}

#[component(modules: [MyModule])]
pub trait MyComponent {

    #[qualified(Foo)]
    fn foo(&self) -> String;
    
    #[qualified(Bar)]
    fn bar(&self) -> String;
    
    fn regular(&self) -> String;
}

pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    assert_eq!(component.foo(), "foo");
    assert_eq!(component.bar(), "bar");
    assert_eq!(component.regular(), "regular");
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
returned trait can only be depended on as  [`Cl<T>`](Cl), and the scoped returned trait or any
objects that depends on it will share the lifetime of the
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

Scoped returned objects are shared and cannot be mutable while they commonly needs mutability. users
must implement internal mutability.

# Metadata

Module additional metadata in the form of
`#[module(key=value, key2=value)]`.

## `subcomponents`

**Optional** path or array of paths to [`#[subcomponent]`](subcomponent) the module should bind. The
subcomponent's builder will be bound with the module, and the subcomponent will have access to all
the bindings of the component/subcomponent the module is installed in.

## `install_in`

**Optional** path to a [`#[define_component]`](define_component)
/[`#[define_subcomponent]`](define_subcomponent) where the module will be automatically installed
in, instead of having to specify the module in a component's [`modules`](component#modules)
metadata.

This allows a module to add bindings to a component that is defined in another crate the current
crate is depending on, For example injecting hooks into a library that will call it.

`install_in` is not allowed on modules with fields, as the component can't understand how to create
the module automatically.