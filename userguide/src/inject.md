# Injecting Objects

Lockjaw can create objects for you, but you need to let lockjaw know how to create the object and
what is needed to create them. These recipes for object creation is called *bindings*, which forms
nodes in a dependency graph, and add edges to other bindings it depends on.

The most simple binding is constructor injection binding.

## Constructor injection

A binding can be created for a struct by using a constructor method, which is a static method
associated to the struct that returns a new instance. The field constructor ( `Foo {a : 0, ...}`) is
not directly used by Lockjaw since methods are more expressive when a none injected field needs a
default value or when transformations are needed on the input.

A struct can be made injectable by marking a struct `impl` block with
the [`#[injectable]`](https://docs.rs/lockjaw/latest/lockjaw/attr.injectable.html) attribute, and
then mark the constructor method
as [`#[inject]`](https://docs.rs/lockjaw/latest/lockjaw/injectable_attributes/attr.inject.html).

```rust,no_run,noplayground
{{#include ../projects/inject/src/lib.rs:foo}}
```

Now Lockjaw understands when trying to create `Foo`, it should call `Foo::new()`.

Note that since it is an associated method, constructor injection only works on a type you *own* (
you can actually change its implementation). For foreign types like imported crates a different
method will be discussed in the [providing objects](provides.md) chapter.

## Constructor dependencies

To create an object, it may many need objects of other types. This is called *dependencies*. In this
example, `Bar` *depends* on having an instance of `Foo` to be created.

In constructor injections, dependencies are listed with its parameters. Lockjaw will try to use
available bindings to create all arguments and pass them to the constructor. If a parameter does not
have a binding, Lockjaw will fail compilation with *missing bindings*.

Note that since we are not asking Lockjaw to actually create the object yet, binding validation
won't be performed. Lockjaw is assuming there are some bindings else where it does not know about
yet.

```rust,no_run,noplayground
{{#include ../projects/inject/src/lib.rs:bar}}
```

If the struct has other fields that can be initialized without injection, like `i`, it can be
directly assigned. If the object needs a runtime value (for example, if `i` needs to be assigned by
the caller with a user input), then [factories](factory.md) will be needed, which will be discussed
later.

## Manual injection

For a moment let's forget about Lockjaw, and try to do dependency injection manually. With the
binding information we have we can write a factory that can create the objects we just defined:

```rust,no_run,noplayground
{{#include ../projects/inject/src/lib.rs:factory}}
```

Note that there is one method for each binding, only taking `&self` and returning the binding type.
Inside the method it calls the constructor method we just marked, and calls other binding methods to
generate the argument.

The factory is an object instead of just methods, since it might need to carry states in the future
(For example, returning a reference to a shared object owned by the factory.)

Writing the factory by hand gets complicated and boring fast. In the next chapter we will ask
Lockjaw to generate it.

[Source](https://github.com/azureblaze/lockjaw/tree/main/userguide/projects/inject/) of this chapter