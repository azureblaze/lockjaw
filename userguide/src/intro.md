# Introduction

Lockjaw is a fully static, compile-time
[dependency injection](https://en.wikipedia.org/wiki/Dependency_injection) framework for
[Rust](https://www.rust-lang.org/) inspired by [Dagger](https://dagger.dev).

## Why use a dependency injection framework

The main purpose of dependency injection is to separate the concerns of creating an object and using
an object. In larger projects creating an object soon becomes a complicated process since creating
an object ofter requires other objects that needs to be created fist(a dependency). Once the object
dependency graph grows deep and wide adding a new edge or node becomes a painful endeavor. For
example, you may need to modify the signature of a dozen methods and interfaces just to pass the new
object all the way to the site it is actually going to be used. This often ends up with
anti-patterns like god objects and global states holding objects so the path can be shortened.

The dependency injection *technique* alleviates this problem by making an object only receive the
objects they need. The object no longer needs to know what is required to create those objects, nor
will have any change if their indirect dependency changes. Responsibility for creating objects is
moved to factories.

A dependency injection *framework* further manages this by automating the creation of the factories.
Users simply specify what each object needs, which implementation to use for an interface, etc., and
the objects can be created.

## Why use Lockjaw as the dependency injection framework

Lockjaw is inspired by [Dagger](https://dagger.dev), which is a mature dependency injection
framework for Java. Lockjaw has feature parity with Dagger (
sans [producers](https://dagger.dev/dev-guide/producers), which may not be too useful in Rust with
async/await available.)

Main features:

* Compile time dependency resolution
    * Lockjaw makes sure all dependencies are fulfilled at compile time. The code will fail to
      compile if a dependency is missing, there are duplicated bindings for the same type, or if the
      dependency graph has cycles. There will be no runtime errors which are harder to detect.
* Cross-crate injection
    * Lockjaw is designed to be used across crates. Clients are able to inject bindings provided by
      libraries if they also use Lockjaw.
* Minimal generated code surface
    * While procedural macros are utilized heavily by Lockjaw, it avoids directly modifying the code
      the attributes macros are placed on. Only a few generated methods are visible to the user.
      This is especially important since most Rust IDEs today does not understand the output of
      procedural macros, a few extra type hints on `let` expressions is enough to make autocomplete
      functional.
* Optional binding, Multibinding, and generated components for plugin systems
    * Lockjaw allows [inversion of control](https://en.wikipedia.org/wiki/Inversion_of_control)
      between library crates and their user. A library is able to define hooks for clients that
      depends on the library to inject. This is especially useful to test multiple clients using a
      library in isolation.

## Why *NOT* use Lockjaw as the dependency injection framework

While the generated code generally is OK-ish, abhorrent techniques are used to generate the code
under the confinement of the current Rust proc-macro system and is extremely fragile.

Please read the [before using](before.md) chapter before using Lockjaw. Lockjaw currently cannot be
recommended for serious work in good conscious. **YOU HAVE BEEN WARNED**
