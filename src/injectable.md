Annotates a struct impl that can be provided to the dependency graph.

```
# use lockjaw::{epilogue, injectable};
# #[macro_use] extern crate lockjaw_processor;
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

# Metadata

Injectables accept additional metadata in the form of
`#[injectable(key=value, key2=value)]`.

## `scope`

**Optional** path to a [`component`], which makes the `injectable` a scoped singleton
under the `component`.

The `injectable` will only be provided in the `component`, and all objects generated from the
same `component` instance will share the same scoped `injecetable` instance. Since it is shared, the
scoped `injectable` can only be depended on as  `&T` or [`Cl<T>`](Cl), and the scoped `injectable`
or any objects that depends on it will share the lifetime of the
`component`.

```
# use lockjaw::{epilogue, injectable};
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