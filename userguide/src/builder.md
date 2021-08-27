# Builder Modules

In the previous chapter the modules are without fields, so Lockjaw can easily create instances of it
(In fact no instance are created since Lockjaw can just call the static methods). However sometimes
we want to be able to affect the values that are bound at runtime, hence need to be able to change
what `#[provides]` does using fields in the module.

```rust,no_run,noplayground
{{#include ../projects/builder/src/lib.rs:module}}
```

Since the `struct` now has fields Lockjaw can no longer automatically create it, the user must
manually pass in the modules when creating the component.

*Implementation note: While Lockjaw can also try to
use [Default](https://doc.rust-lang.org/std/default/trait.Default.html) or some other mechanisms,
usages like this implies the module has mutable state and generally is a bad idea.*

## Using builder modules

Instead of passing the runtime modules to the component one by one, they are collected in a single
`struct` annotated by
the [`#[builder_modules]`](https://docs.rs/lockjaw/latest/lockjaw/attr.builder_modules.html)
attribute.

```rust,no_run,noplayground
{{#include ../projects/builder/src/lib.rs:builder_modules}}
```

Every field in the struct should be a module. Using a `struct` makes sure each module will be
required by the compiler/IDE while exposing the least amount of generated code(which is harder for
users and IDEs to understand, it is better to spell everything out in visible code.).

The `#[builder_modules]` can then be installed in the component using
the [`builder_modules` metadata](https://docs.rs/lockjaw/latest/lockjaw/attr.component.html#builder_modules)

```rust,no_run,noplayground
{{#include ../projects/builder/src/lib.rs:component}}
```

The component only accepts one`#[builder_modules]`, which is likely to be specifically tailored for
the component. The modules itself can be shared.

The `builder_modules` metadata can be used at the same time with
the [`modules` metadata](https://docs.rs/lockjaw/latest/lockjaw/attr.component.html#modules).
`modules` should be preferred whenever possible as they are easier to use.

## Creating components with builder modules

If
the [`builder_modules` metadata](https://docs.rs/lockjaw/latest/lockjaw/attr.component.html#builder_modules)
is specified,
the  [`#[builder_modules] struct`](https://docs.rs/lockjaw/latest/lockjaw/attr.builder_modules.html)
will become the parameter for
the [`build()` method](https://docs.rs/lockjaw/latest/lockjaw/attr.component.html#component-builder)
of the component which the user must pass.

```rust,no_run,noplayground
{{#include ../projects/builder/src/lib.rs:main}}
```

[Source](https://github.com/azureblaze/lockjaw/tree/main/userguide/projects/builder/) of this
chapter