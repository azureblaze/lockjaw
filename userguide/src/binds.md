# Binding traits

## `#[provides]` trait

A `trait` can be provided using
the [`#[provides]`](https://docs.rs/lockjaw/latest/lockjaw/module_attributes/attr.provides.html)
binding.

```rust,no_run,noplayground
{{#include ../projects/binds/src/main.rs:provides_trait}}
```

However, Lockjaw is going to be particular when trying to request a trait object from the dependency
graph. The concrete implementation of the trait may contain [reference to the component](scoped.md),
but ideally this is not something the consumer of the trait should care about, so Lockjaw enforces
that any trait it provides must not outlive the component. The worst case `'ComponentLifetime` is
assumed, so consumers don't have to change when it actually happens.

The `Box` returned by the component must be bound by the component's lifetime(same as `self`).

```rust,no_run,noplayground
{{#include ../projects/binds/src/main.rs:provides_component}}
```

## `#[binds]` trait

While `#[provides]` kind of works, binding an implementation to a `trait` interface is a common
operation so Lockjaw has
the [`#[binds]`](https://docs.rs/lockjaw/latest/lockjaw/module_attributes/attr.binds.html) attribute
to make them easier to use.

For an interface and an implementation:

```rust,no_run,noplayground
{{#include ../projects/binds/src/main.rs:logger}}
```

`#[binds]` can be used to create binding that says "when the `Logger` interface is needed,
use `StdoutLogger` as the actual implementation":

```rust,no_run,noplayground
{{#include ../projects/binds/src/main.rs:binds}}
```

The method body must be empty, as Lockjaw will replace it.

The[`Cl`](https://docs.rs/lockjaw/latest/lockjaw/enum.Cl.html) in the return type means
**C**omponent **l**ifetimed, which is a wrapper around a type forcing it not outlive the component.
Having this wrapper makes it easier for the compiler to deduce the lifetime.

With the binding defined the `Logger` can now be used by other classes, without caring about the
actual implementation.

```rust,no_run,noplayground
{{#include ../projects/binds/src/main.rs:greeter}}
```

Note that `Logger` still has to be injected as `Cl<dyn Logger>`, and `Greeter` is also bound by the
lifetime of the component.

## Unit testing with dependency injection

`StdoutLogger` writes its output straight to the console, so it is hard to verify `Greeter` actually
sends the correct thing. While we can give `StdoutLogger` special apis to memorize what it logs and
give access to tests, having test code in prod is generally bad practice.

Instead we can use dependency injection to replace the environment `Greeter` runs in. We can create
a `TestLogger` that writes the logs to memory and can read it later, bind it to the `Logger` with a
module, and install the module in a component for test that has all test bindings. We are than able
to test `Greeter` without adding test code to the `Greeter` itself:

```rust,no_run,noplayground
{{#include ../projects/binds/src/main.rs:test}}
```

Generally, a library should also provide a test implementation and a module that binds the test
implementation. The consumer of the library can then test by installing the test module instead of
the real module. This allows test infrastructure to easily be shared. Some kind of test scaffolding
can also be created to auto generate the component and inject objects into tests, but that is out of
scope for lockjaw itself.

Note that in the `TestLogger` `MESSAGES` is a static mutable, since `log()` and `get_messages()` is
going to be called on different instances of `TestLogger`. This is `unsafe` and bad, so in the next
chapter we will discuss how to handle this by forcing a single instance of `TestLogger` to be shared
among everything that uses it.

[Source](https://github.com/azureblaze/lockjaw/tree/main/userguide/projects/binds/) of this chapter