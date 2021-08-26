# Empty multibindings

The binding for `Vec<T>` and `HashMap<K,V>` is automatically generated when [`#[into_vec]`](vec.md)
or [`#[into_map]`](map.md) is encountered. However when such binding does not exist and someone
depended on the collection, Lockjaw cannot be sure if it should provide an empty collection since it
should be a multibinding, or if the user forgot to bind the collection.

This usually happens when a library defines a multibinding for events, etc., but does not bind
anything to it itself, and clients aren't forced to use the event.

A [`#[multibinds]`](https://docs.rs/lockjaw/0.2.0/lockjaw/module_attributes/attr.multibinds.html)
method that returns the collection type should be declared in such case, to let Lockjaw know a
multibinding collection is intended, but it may be empty.

```rust,no_run,noplayground
{{#include ../../tests/module_multibinds.rs:multibinds}}
```

`#[multibinds]` also serves as documentation on the `#[module]` specifying it is expecting the
multibinding.