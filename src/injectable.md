Annotates a struct impl that can be provided to the dependency graph.

```
# use lockjaw::{epilogue, injectable};
# #[macro_use] extern crate lockjaw_processor;
# lockjaw::prologue!("src/lib.rs");
struct Bar{}

#[injectable]
impl Bar {
    #[inject]
    pub fn new() -> Self {
        Self {}
    }
}

struct Foo{
    bar : crate::Bar,
    s : String,
}

#[injectable]
impl Foo {
    #[inject]
    pub fn new(bar : crate::Bar,) -> Self {
        Self {bar, s: "foo".to_owned()}
    }
}

#[component]
trait MyComponent {
    fn foo(&self) -> crate::Foo;
}

pub fn main() {
    let component = <dyn MyComponent>::new();
    let foo = component.foo();
}
epilogue!();
```

# Method attributes

Exactly one method under the `#[injectable]` should have a constructor attribute:

* [`#[inject]`](injectable_attributes::inject)
* [`#[factory]`](injectable_attributes::factory)

These attributes designate which method lockjaw should call to create a instance.

Method attributes are nested under `#[injectable]`, and all nested attributes should be
unqualified (always used as `#[attribute]` instead of `#[lockjaw::attribute]`).

### Generated code

For a struct `Foo` with a `[factory]` method called `create_foo`:

```ignore
struct Foo {...}

#[injectable]
impl Foo {
   #[factory]
   pub fn create_foo(#[runtime] runtime_1: Type1, injected: Injected) -> Foo {
      ...
   }
}
```

The struct `FooFactory<'component>` will be generated at the same module.

```ignore
struct FooFactory<'component> { ... }

impl FooFactory<'_> {
   pub fn create_foo(&self, runtime_1 : Type1) -> Foo {
      Foo::create_foo(runtime_1, self.injected.get())
   }
}
```

The factory contains a method with the same name as the `#[factory]` method. The factory can
implement a trait instead by using the [`implementing`](#implementing) metadata.

The factory depend on bindings from the component, hence cannot outlive it.

`FooFactory` has private visibility by default, which can be overridden by using the
[`visibility`](#visibility) metadata.

### Metadata

Injectable factories accept additional metadata in the form of
`#[factory(key=value, key2=value)]`.

#### `implementing`

**Optional** path to a trait which the factory will implement, instead of adding a method to the
factory struct.

For a struct `Foo` , the factory trait should have a method with the signature of
`fn create_foo(&self, runtime_parameters, ...) -> Foo`. The name of the `#[factory]` method should
match the trait method.

```
# use lockjaw::{epilogue, injectable, module, component, Cl};
# lockjaw::prologue!("src/lib.rs");
pub struct Foo {
    pub i: i32,
    pub phrase: String,
}

pub trait FooCreator {
  fn create_foo(&self, i: i32) -> Foo;
}

#[injectable]
impl Foo {
    #[factory(implementing: FooCreator)]
    fn create_foo(#[runtime] i: i32, phrase: String) -> Self {
        Self { i, phrase }
    }
}


struct MyModule;

#[module]
impl MyModule {
    #[provides]
    pub fn provide_string(&self) -> String {
        "helloworld".to_owned()
    }
    
    #[binds]
    pub fn bind_foo_creator(impl_: FooFactory) -> Cl<dyn FooCreator> {}
}

#[component(modules: MyModule)]
pub trait MyComponent {
    fn foo_creator(&self) -> Cl<dyn FooCreator>;
}

pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::build();

    let foo = component.foo_creator().create_foo(42);

    assert_eq!(foo.i, 42);
    assert_eq!(foo.phrase, "helloworld");
}

epilogue!();
```

#### `visibility`

**Optional** string specifying the visibility of the generated factory. The string must conform to
the [rust visibility syntax](https://doc.rust-lang.org/reference/visibility-and-privacy.html), e.g.
`"pub"`, `"pub(crate)"`, `"pub(super)"`, or `"pub(in path::to::mod)"`

The factory is private by default.

# Metadata

Injectables accept additional metadata in the form of
`#[injectable(key=value, key2=value)]`.

## `scope`

**Optional** path to a [`component`](component), which makes the `injectable` a scoped singleton
under the `component`.

The `injectable` will only be provided in the `component`, and all objects generated from the
same `component` instance will share the same scoped `injecetable` instance. Since it is shared, the
scoped `injectable` can only be depended on as  `&T` or [`Cl<T>`](Cl), and the scoped `injectable`
or any objects that depends on it will share the lifetime of the
`component`.

```
# use lockjaw::{epilogue, injectable};
# lockjaw::prologue!("src/lib.rs");
pub struct Foo {}

#[injectable(scope : crate::MyComponent)]
impl Foo {
    #[inject]
    pub fn new()-> Self {
        Self {}
    }
}

pub struct Bar<'a>{
    foo : &'a crate::Foo
}

#[injectable]
impl Bar<'_> {
    #[inject]
    pub fn new(foo : &'_ crate::Foo) -> Bar<'_> {
        Bar {foo}
    }
}

#[lockjaw::component]
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

Scoped `injectables` are shared and cannot be mutable while they commonly needs mutability. users
must implement internal mutability.

## `container`

**Optional** Specifies a container such as `RefCell<T>` to place the `injectable` in. The metadata
is only applicable when [`scope`](#scope) is also used.

The `container` must be a generic in the form of `CONTAINER<T>`, which has a
`pub fn new(value: T) -> CONTAINER<T>` method. Most common rust containers like `Cell`, `Rc` are
applicable.

When `container` is specified, the non-contained form of the struct can not be injected.

Typically, this is used to give internal mutability to a shared instance.

```
# use lockjaw::{epilogue, injectable, component};
# use std::cell::RefCell;
# lockjaw::prologue!("src/lib.rs");

pub struct Foo {
    pub i: u32,
}

#[injectable(scope: MyComponent, container: RefCell)]
impl Foo {
    #[inject]
    pub fn new() -> Self {
        Self {
            i: Default::default(),
        }
    }

    pub fn count(&mut self) -> u32 {
        self.i = self.i + 1;
        self.i
    }
}

#[component]
pub trait MyComponent {
    fn foo(&self) -> &RefCell<crate::Foo>;
}

pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    let foo1 = component.foo();
    let foo2 = component.foo();

    assert_eq!(foo1.borrow_mut().count(), 1);
    assert_eq!(foo1.borrow_mut().count(), 2);
    assert_eq!(foo2.borrow_mut().count(), 3);
}
epilogue!();
```