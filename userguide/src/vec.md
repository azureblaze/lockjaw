# `Vec<T>` multibindings

A [`#[provides]`](https://docs.rs/lockjaw/latest/lockjaw/module_attributes/attr.provides.html) or
[`#[binds]`](https://docs.rs/lockjaw/latest/lockjaw/module_attributes/attr.binds.html) binding can
also be marked with
the [`#[into_vec]`](https://docs.rs/lockjaw/latest/lockjaw/module_attributes/attr.into_vec.html)
attribute, which means instead of directly binding to `T`, the binding should be collected into a
`Vec<T>`.

With the bindings:

```rust,no_run,noplayground
{{#include ../../integration_tests/tests/module_provides_into_vec.rs:into_vec}}
```

`Vec<String>` can be injected with the values `["string1", "string2"]`. This works across all
modules that are installed.

`#[into_vec]` can also be [`#[qualified]`](qualifiers.md)

```rust,no_run,noplayground
{{#include ../../integration_tests/tests/module_provides_into_vec.rs:qualified}}
```

Which result in `#[qualified(Q)] Vec<String>`. Note that the container is qualified instead of the
content.

`#[into_vec]` also works with [`#[binds]`](binds.md)

```rust,no_run,noplayground
{{#include ../../integration_tests/tests/module_provides_into_vec.rs:binds}}
```

Which allows `Vec<Cl<dyn Foo>>` to be injected. This is a common way to implement event callbacks.

## Providing multiple items

A method marked
with [`#[elements_into_vec]`](https://docs.rs/lockjaw/latest/lockjaw/module_attributes/attr.elements_into_vec.html)
can return `Vec<T>`, which will get merged with other `#[into_vec]` and `#[elements_into_vec]`.

```rust,no_run,noplayground
{{#include ../../integration_tests/tests/module_provides_into_vec.rs:elements_into_vec}}
```

This allows multiple bindings to be provided at once. It also allows a binding method to decide not
to provide anything at runtime, by returning an empty `Vec`.

## Duplication behaviors

Lockjaw's `#[into_vec]` strays from
Dagger's [`@IntoSet`](https://dagger.dev/api/latest/dagger/multibindings/IntoSet.html), as `Hash`
and `Eq` are not universally implemented in Rust. This mean the `Vec` may contain duplicated values.

However, duplicated modules are not allowed by Lockjaw, so each binding method will be called at
most once when generating the `Vec`.

If deduplication is needed, you can add another provider that does the conversion:

```rust,no_run,noplayground
#[provides]
pub fn vec_string_to_set_string(v: Vec<String>) -> HashSet<String> {
    HashSet::from_iter(v)
}
```

## Examples

https://github.com/azureblaze/lockjaw/blob/main/tests/module_provides_into_vec.rs