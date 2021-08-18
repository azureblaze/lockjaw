Injections does not always work:

* Traits are not structs and can not be instantiated.
* `#[injectable]` and `#[inject]` cannot annotate third party structs.
* The desired implementation of a trait may be affected by run time conditions, like switching
  implementations based on reading a config file.

For these cases, a `#[provides]` method can be used instead. A `#[provides]` method satisfy its
return type. When the type is requested the method is invoked.

```rust
#[provides]
pub fn provide_string() -> String {
    "my_string".to_owned()
}
```

Whenever someone requested a `String`, they will get a `String` with the value `"my_string"`.

`#[provides]` method can also have their own dependencies, which is requested through the method
parameter. Lockjaw will satisfy them first and pass them to the method.

```rust
pub struct Foo {
    value: String
}

#[provides]
pub fn provide_foo(s : String) -> crate::Foo {
    Foo {value: s}
}
```

When someone requested a `Foo`, `provided_foo()` will be invoked with the `String` returned by
`provided_string()`, and used to create the `Foo` struct.

`#[provides]` can also be used for traits:

```rust
pub trait Foo {
    fn foo();
}

pub struct FooImpl {}

#[injectable]
impl FooImpl {
    #[inject]
    pub fn new() -> Self {
        Self {}
    }
}

impl Foo for FooImpl {
    fn foo() {
        //...
    }
}

#[provides]
pub fn provide_foo(impl_ : FooImpl) -> Box<dyn Foo> {
    Box::new(impl_)
}
```

We can also simplify trait provisions with a `#[binds]` method:

```rust
#[binds]
pub fn bind_foo_impl(impl_ : FooImpl) -> Cl<dyn Foo> {}
```

Note that since the implementation may contain a reference to a singleton (more
on [scope](#scoped-bindings)
later), the return type is `Cl<T>`. `Cl<T>` (ComponentLifetime) forces the returned value to have a
lifetime less than the component itself.

Since `#[provides]` and `#[binds]` are less coupled with the type they provide, and users might want
to swap out implementations (like use a `FakeClient` for test that emulates talking to a server
without actual network operations), these bindings should not be global. They are grouped into
a `#[module]` so they can be incorporated later into a specific dependency graph. `#[module]`
annotates a [impl block](https://doc.rust-lang.org/std/keyword.impl.html) of a struct to define the
bindings.

```rust
struct MyModule {}
#[module]
impl MyModule {
    #[binds]
    pub fn bind_foo_impl(impl_ : FooImpl) -> impl Foo {}
}
```

Note: in lockjaw documentations, "modules" always refer to the dependency injection module. The Rust
module is always referred as `mod`.