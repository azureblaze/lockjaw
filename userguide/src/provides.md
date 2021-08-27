## Providing Objects

In this chapter we will discuss another way to create bindings.

## Limitations of constructor injection bindings

Constructor injections are designed to be the **only** way to create an instance of the type it
binds. It is even considered bad practice to call the constructor manually. Hence, constructor
injections has some limitations:

* Constructor injections can only be done by owned types (can only be defined by the `mod` that
  defines the type itself.).
  * If you don't own the type you should not say something is the only way to create it.
* Can only create concrete types
  * Sometimes you may want to bind traits and swap the implementation, maybe at runtime.

## Modules

Obviously Lockjaw is not going to ask the world to use it or the user to rewrite everything they use
with it, so it gives other ways to bind types. Since these bindings are no longer the "one true way
to create things", and different bindings for the same type may be needed within the same program,
the user needs to be able to select which bindings to use in each dependency graph.

In Lockjaw, these elective bindings are defined in *modules*, and the component can choose what
modules to *install*, which imports its bindings. Note that in Lockjaw documentation *modules*
always refer dependency injection modules, and `mod` will be used to refer to Rust modules.

To declare a module with Lockjaw
the [`#[module]`](file:///C:/git/lockjaw/target/doc/lockjaw/attr.module.html) attribute should be
used to mark the `impl` block of a struct.

```rust,no_run,noplayground
struct MyModule {}

#[module]
impl My Module {
  ...
}
```

The `impl` block will contain the binding definitions.

For now the modules should be static (without fields). Modules with fields will be discussed
in [builder modules](builder.md)

## #[provides] bindings

The [`#[provides]`](https://docs.rs/lockjaw/latest/lockjaw/module_attributes/attr.provides.html)
binding annotates a method that returns the type.

```rust,no_run,noplayground
{{#include ../projects/provide/src/lib.rs:provides}}
```

Like [`#[inject]`](https://docs.rs/lockjaw/latest/lockjaw/injectable_attributes/attr.inject.html),
the `#[provides]` method can also request other bindings from the dependency graph, and produce the
target value with it.

```rust,no_run,noplayground
{{#include ../projects/provide/src/lib.rs:provides_with_dep}}
```

## Installing modules

`#[module]` on its own is just a collection of bindings and does not do anything. It must be
installed in a [`#[component]`](https://docs.rs/lockjaw/latest/lockjaw/attr.component.html) to joint
the dependency graph. This is done by listing the module type in
the [`modules` metadata](https://docs.rs/lockjaw/latest/lockjaw/attr.component.html#modules) of the
component.

```rust,no_run,noplayground
{{#include ../projects/provide/src/lib.rs:component}}
```

A lot of Lockjaw attribute macros also takes *metadata arguments*, which is comma
separated `key : value` pairs in a parenthesis. The values are usually string literal, integers,
types, arrays of values (`foo : [value 1, value 2]`), or more metadata (`foo : { key : value }`). In
this case `modules` takes an array of types (of `#[modules]`).

Providing `trait` is a bit more complicated and will be discussed [later](binds.md).

[Source](https://github.com/azureblaze/lockjaw/tree/main/userguide/projects/provide/) of this
chapter