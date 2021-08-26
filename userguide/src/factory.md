# Factory

Sometimes an object needs not only injected values but also runtime values to be created, such as
constructor parameters that depends on user input.

This can be handled by writing a factory that injects bindings as a [Provider](provider.md), and
combine it with the runtime value to create the object. For example,

```rust,no_run,noplayground
pub struct Foo {
    pub i: i32, // runtime
    pub s: String, // injected
}

pub struct FooFactory<'component>{
    s_provider: Provider<'component, String>
}

#[injectable]
impl FooFacotory<'_> {
    #[inject]
    pub fn new(s_provider: Provider<String>) -> FooFactory {
       FooFactory { s_provider }
    }
    
    pub fn create(&self, i: i32) -> Foo {
       Foo { i, s : self.s_provider.get() }
    }
}
```

This is a lot of boilerplate, and can be automated by
using [`#[factory]`](https://docs.rs/lockjaw/0.2.0/lockjaw/injectable_attributes/attr.factory.html)
instead
of [`#[inject]`](https://docs.rs/lockjaw/0.2.0/lockjaw/injectable_attributes/attr.inject.html)

```rust,no_run,noplayground
{{#include ../../tests/injectable_factory.rs:factory}}
```

Runtime parameters needs to be marked with
the [`#[runtime]`](https://docs.rs/lockjaw/0.2.0/lockjaw/injectable_attributes/attr.factory.html#runtime)
attribute.

`FooFactory` will be created by Lockjaw, with a method with the same name as the marked method
taking only runtime parameters.

```rust,no_run,noplayground
{{#include ../../tests/injectable_factory.rs:factory_use}}
```

## Factory traits

The factory can also be instructed to implement a `trait` by using the
[`implementing` metadata](https://docs.rs/lockjaw/0.2.0/lockjaw/injectable_attributes/attr.factory.html#implementing)
.

```rust,no_run,noplayground
{{#include ../../tests/injectable_factory_implementing.rs:factory}}
```

The method name and runtime signature must match the `trait` method the factory should override.

This is especially useful to bind the factory to a trait

```rust,no_run,noplayground
{{#include ../../tests/injectable_factory_implementing.rs:bind}}
```

## Examples

https://github.com/azureblaze/lockjaw/blob/main/tests/injectable_factory.rs
https://github.com/azureblaze/lockjaw/blob/main/tests/injectable_factory_implementing.rs