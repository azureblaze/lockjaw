# Subcomponents

[Scopes](scoped.md) enforces an object to be a singleton within the component, so everyone can
interact with the same instance. However there are times when we need multiple sets of such
instances. For example, we might be implementing a web server, and there are a lot of shared
resources with every session. We want the session to be independent, so we need to create component
for each session:

```rust,no_run,noplayground
impl Server {
    pub fn new_session(url: Url) -> Session {
      Session{
         component: <dyn SessionComponent>::build({url,...})
      }
    }
}
```

However, there are also some resource that belongs to the whole server shared by all sessions, for
example, an IO bound thread pool. We have to rebind this in every `SessionComponent`

```rust,no_run,noplayground
impl Server{
    #[inject]
    pub fn new(#[qualified(IoBound)] io_threadpool: &ThreadPool) { ... }
    
        pub fn new_session(&self, url: Url) -> Session {
        Session {
            component: <dyn SessionComponent>::build({
                url,
                self.io_threadpool,
                ...
            })
        }
    }
}
```

Managing these will soon get ugly.

Instead we can create
a [`#[subcomponent]`](https://docs.rs/lockjaw/latest/lockjaw/attr.subcomponent.html) which can have
distinct scoped bindings, modules, but also has access to bindings in its parent component.

Using `#[subcomponent]` is almost identical to a regular `#[component]`, except that the
`#[subcomponent]` has to be installed in a parent component, and the method to create an instance.

## Installing a `#[subcomponent]`

A `#[subcomponent]` is installed by first listing it in
a [`#[module]`](https://docs.rs/lockjaw/latest/lockjaw/attr.module.html#) using
the [`subcomponents` metadata](https://docs.rs/lockjaw/latest/lockjaw/attr.module.html#subcomponents)

```rust,no_run,noplayground
{{#include ../../integration_tests/tests/sub_component.rs:list}}
```

Then installing the `#[module]` in a parent component. The parent component can be either a regular
component or another subcomponent.

## Creating a instance of the subcomponent

The `#[module]` with a `subcomponents: [FooSubcomponent]` metadata creates hidden binding
of `Cl<FooSubcomponentBuilder>` which can be injected to create new instances of `FooSubcomponent`
by calling `build()`. `build()` also takes the
corresponding [`#[builder_modules]`](https://docs.rs/lockjaw/latest/lockjaw/attr.builder_modules.html)
if the subcomponent is defined with
the [`builder_modules` metadata](https://docs.rs/lockjaw/latest/lockjaw/attr.component.html#builder_modules)

`build` can be called multiple times to create independent subcomponents, with the parent being
shared.

## Lifetime

The lifetime of the subcomponent is bound by its parent.

## Examples

https://github.com/azureblaze/lockjaw/blob/main/tests/sub_component.rs