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

# Methods

## `#[inject]` methods
Denotes the method as "injection constructor", which is the method lockjaw will call to create
the object.

One and only one method must be annotated with `#[inject]` in an `#[injectable]` struct. The
method must be static, and must return an instance of the struct.

The method can request other injectable objects with its parameters. Lockjaw will fulfil those
objects before calling the injection constructor.

# Metadata

Injectables accept addtional metadata in the form of
`#[injectable(key="value", key2="value2")]`. Currently all values are string literals.

## `path`
**Optional** [path](https://doc.rust-lang.org/reference/paths.html) relative to the path of the
current file.

Lockjaw retrieves the path of the current file from [`epilogue!()`](epilogue) and
[`mod_epilogue!()`](mod_epilogue), but if the `injectable` is nested under a
[`mod`](https://doc.rust-lang.org/reference/items/modules.html) then the extra path must be
specified.

```compile_fail
# use lockjaw::{epilogue, injectable};
# lockjaw::prologue!("src/lib.rs");

mod nested {
    pub struct Foo {}
    #[lockjaw::injectable(path = "nested")]
    impl Foo {
        #[inject]
        pub fn new()-> Self {
            Self {}
        }
    }
}
#[lockjaw::component]
pub trait MyComponent {
    fn foo(&self) -> crate::nested::Foo;
}

pub fn main() {
    let component = <dyn MyComponent>::new();
    component.foo();
}
epilogue!();
```
## `scope`

**Optional** fully qualified path to a [`component`](component), which makes the `injectable` a
scoped singleton under the `component`.

The `injectable` will only be provided in the `component`, and all objects generated from the
same `component` instance will share the same scoped `injecetable` instance. Since it is shared,
the scoped `injectable` can only be depended on as  `&T` or [`ComponentLifetime<T>`](ComponentLifetime), and
the scoped `injectable` or any objects that depends on it will share the lifetime of the
`component`.

```
# use lockjaw::{epilogue, injectable};
# lockjaw::prologue!("src/lib.rs");
pub struct Foo {}

#[injectable(scope = "crate::MyComponent")]
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

Scoped `injectables` are shared and cannot be mutable while they commonly needs mutability.
users must implement internal mutability.