# Optional bindings

Sometimes a binding might be optional, being behind
a [cargo feature](https://doc.rust-lang.org/cargo/reference/features.html) or provided by an
optional library. It will be useful to allow such bindings to be missing from the dependency graph,
and detect whether such binding exists.

In Lockjaw a binding can be declared as optional by using
the [#[binds_option_of]](https://docs.rs/lockjaw/0.2.0/lockjaw/module_attributes/attr.binds_option_of.html)
method attribute in a [#[module]](https://docs.rs/lockjaw/0.2.0/lockjaw/attr.module.html)

```rust,no_run,noplayground
{{#include ../../tests/module_binds_option_of.rs:binds}}
```

The `#[binds_option_of]` method should take no parameter and return the type `T` to bind
as [`Option<T>`](https://doc.rust-lang.org/std/option/enum.Option.html). This does not actually bind
the `T`.

```rust,no_run,noplayground
{{#include ../../tests/module_binds_option_of.rs:component}}
```

If `T` is actually bound somewhere else, injecting `Option<T>` will result in `Some(T)`. Otherwise
it will be `None`.