# Scoped Bindings

By default everytime a dependency needs to be satisfied, lockjaw creates a new instance, and move it
to the dependency (field or method parameter). This is not always desired since an object may be
used to carry some common state, and we want every type that depends on it to get a reference to a
single instance instead (singletons).

In the last chapter, we had to use a mutable static state to store the messages `TestLogger` logs,
since we need to read the same messages later but a different instance of `TestLogger` will be
created.

To do this, the [`scope` metadata](https://docs.rs/lockjaw/latest/lockjaw/attr.injectable.html#scope)
can be specified on a `#[injecatable]` or `#[provides]`
, passing a component's path. This means there are only one instance of the type for objects created
by the same instance of component (they are not global singletons, you can still have multiple
instances if you have multiple components).

```rust,no_run,noplayground
{{#include ../projects/scoped/src/main.rs:test_logger}}
```

Other types can depend on a scoped type as a reference (`&T`) or `Cl<T>`

```rust,no_run,noplayground
{{#include ../projects/scoped/src/main.rs:component}}
```

Although `#[binds]` has to explicitly ask for `&TestLogger`

```rust,no_run,noplayground
{{#include ../projects/scoped/src/main.rs:binds}}
```

Note that `Greeter` hasn't changed at
all. [`Cl<T>`](https://docs.rs/lockjaw/latest/lockjaw/enum.Cl.html) allows a type to decouple itself
from whether the type depended on is scoped or not. It may be an owned instance or a shared
instance, but the type does not care as it will not try to move it.

## Lifetime

Scoped objects are owned by the component and has the same lifetime as it.

## Handling mutability

In most uses a scoped type probably should be mutable to make it useful. However we cannot request
it as `&mut T` since certainly multiple objects will try to request it. Scoped types must
implement [interior mutability](https://doc.rust-lang.org/reference/interior-mutability.html) itself
and use an immutable interface. In the example `TestLogger` use
a [`RefCell`](https://doc.rust-lang.org/std/cell/struct.RefCell.html) so the messages can be mutated
even when the `TestLogger` itself is immutable.

Sometimes it might be easier to wrap the whole class in a memory container like a `RefCell`
or [`RwLock`](https://doc.rust-lang.org/std/sync/struct.RwLock.html).
The [`container` metadata](https://docs.rs/lockjaw/latest/lockjaw/attr.injectable.html#container)
can be used on a [`#[injectable]`](https://docs.rs/lockjaw/latest/lockjaw/attr.injectable.html) to
bind the type as `&CONTAINER<T>` instead of `&T` 