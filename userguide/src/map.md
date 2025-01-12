# `HashMap<K,V>` multibinding

Similar to [`Vec<T> multibinding`](vec.md),
a [`#[provides]`](https://docs.rs/lockjaw/latest/lockjaw/module_attributes/attr.provides.html) or
[`#[binds]`](https://docs.rs/lockjaw/latest/lockjaw/module_attributes/attr.binds.html) binding can
also be marked with
the [`#[into_map]`](https://docs.rs/lockjaw/latest/lockjaw/module_attributes/attr.into_map.html)
attribute, which collects the key-value pair into
a [`HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html)

While a map can be created by multibinding `Vec<(K,V)>` or some other entry generating mechanisms,
the `HashMap<K,V>` multibinding has additional compile time checks to make sure there are no key
collisions.

## Map keys

The value type is specified by the binding method return value, but the key type and value needs to
be specified by a metadata in the `#[into_map]` attribute.

### `string_key`

`string_key` specifies the map key is
a [`String`](https://doc.rust-lang.org/std/string/struct.String.html). The value must be a string
literal, lockjaw is unable to resolve more complex compile time constants.

This example binds to `HashMap<String,String>`:

```rust,no_run,noplayground
{{#include ../../integration_tests/tests/module_provides_into_map.rs:string_key}}
```

### `i32_key`

`i32_key` specifies the map key is an `i32`. The value must be an i32 literal, lockjaw is unable to
resolve more complex compile time constants.

This example binds to `HashMap<i32,String>`:

```rust,no_run,noplayground
{{#include ../../integration_tests/tests/module_provides_into_map.rs:i32_key}}
```

Other types are not implemented. `i32` ought to be enough for everyone.

### `enum_key`

`i32_key` specifies the map key is an `enum`. Since the `enum` is going to be used as the map key,
it must satisfy the same constraints `HashMap` gives, which is
implementing [`Eq`](https://doc.rust-lang.org/std/cmp/trait.Eq.html)
and [`Hash`](https://doc.rust-lang.org/std/hash/trait.Hash.html). It also must be a simple enum with
no fields so Lockjaw knows how to compare them at compile time (meaning comparing the name is
enough).

```rust,no_run,noplayground
{{#include ../../integration_tests/tests/module_provides_into_map.rs:enum}}
```

This example binds to `HashMap<E,String>`:

```rust,no_run,noplayground
{{#include ../../integration_tests/tests/module_provides_into_map.rs:enum_key}}
```

Lockjaw is able to infer the enum type (`E`) if the value is imported (`use E::Bar`), but the code
maybe be more readable if the type is explicitly spelled out, especially most IDEs today cannot
properly inspect tokens inside the metadata.

## Qualifiers

`#[into_map]` can also be [`#[qualified]`](qualifiers.md)

```rust,no_run,noplayground
{{#include ../../integration_tests/tests/module_provides_into_map.rs:qualified}}
```

Which result in ` #[qualified(Q)] HashMap<String, String>`. Note that the container is qualified
instead of the content.

## Dynamic map entries

All bindings in `#[into_map]` must be resolved at compile time, There are
no [`#[elements_into_vec]`](https://docs.rs/lockjaw/latest/lockjaw/module_attributes/attr.elements_into_vec.html)
equivalent such as `#[elements_into_map]`.

However dynamic map entries can be achieved by rebinding `Vec<(K,V)>` into a `HashMap<K,V>`.

## Examples

https://github.com/azureblaze/lockjaw/blob/main/tests/module_provides_into_map.rs