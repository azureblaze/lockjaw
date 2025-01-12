# Defined components

One of the issues with
using [`#[component]`](https://docs.rs/lockjaw/latest/lockjaw/attr.component.html#)
and [`#[subcomponent]`](https://docs.rs/lockjaw/latest/lockjaw/attr.subcomponent.html) is that
modules still has to be listed, which means anything using the component will depend on everything.
The component is also generated in the crate, so other crates depending on it is not able to expand
the dependency graph, which makes [multibindings](multibindings.md) less useful. Additionally, unit
tests often needs a different set of modules, so the whole component has to be redefined.

In a large project there maybe tens and even hundreds of modules, and this will become very
difficult to manage.

Instead of `#[component]` and `#[subcomponent]`, Lockjaw also
provides [`#[define_component]`](https://docs.rs/lockjaw/latest/lockjaw/attr.define_component.html)
and[`#[define_subcomponent]`](https://docs.rs/lockjaw/latest/lockjaw/attr.define_subcomponent.html)
which automatically collects modules from the entire build dependency tree, so they no longer need
to be manually installed.

## Automatically installing modules

[`#[modules]`](https://docs.rs/lockjaw/latest/lockjaw/attr.module.html) can be automatically
installed in a component by using
the [`install_in` metadata](https://docs.rs/lockjaw/latest/lockjaw/attr.module.html#install_in). The
metadata takes a path to a `#[define_component]` `trait`. Alternatively, it can also be a path to
[`Singleton`](https://docs.rs/lockjaw/latest/lockjaw/trait.Singleton.html), which means it should be
installed in every `#[define_component]` but not `#[define_subcomponent]`.

Such modules cannot have fields.

```rust,no_run,noplayground
{{#include ../../integration_tests/tests/module_install_in.rs:install_in}}
```

## Entry points

Ideally a component should only be used at the program's entry point, and rest of the program should
all use dependency injection, instead of trying to pass the component around. However sometimes
callbacks will be called from non-injected context, and the user will need to reach back into the
component.

These kinds of usage will cause the [requesting methods](request.md) in a component to bloat, and
add redundant dependencies or cycle issues to everyone that uses the component.

With `#[define_component]`
, [`#[entry_point]`](https://docs.rs/lockjaw/latest/lockjaw/attr.entry_point.html) can be used.

An `#[entry_point]` has binding requesting methods just like a component.
The [`install_in` metadata]() needs to be used to install the `#[entry_point]` in a component. Once
installed, the

```rust,no_run,noplayground
<dyn FooEntryPoint>::get(component : &dyn FooComponent) -> &dyn FooEntryPoint
```

method can be used to cast the opaque component into the entry point, and access the dependency
graph.

```rust,no_run,noplayground
{{#include ../../integration_tests/tests/entry_point.rs:entry_point}}
```

## Testing with `#[define_component]`

While compiling tests, Lockjaw gathers `install_in` modules only from the `[dev-dependencies]`
section of `Cargo.toml` instead of the regular `[dependencies]`, even though `[dev-dependencies]`
inherits `[dependencies]`. This is due to tests often have conflicting modules with prod code. any
prod modules that need to be used in tests has to be relisted again in the `[dev-dependencies]`
section. 