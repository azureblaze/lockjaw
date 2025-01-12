# `Provider<T>`

Normally when injecting a dependency, *one* instance of the dependency is *created before* creating
the depending object. This may not be ideal since the depending object might:

* Want multiple instance of the object, for example, populating an array.
* Have cyclic dependency at **runtime**

For every binding `T`, Lockjaw also automatically creates a binding
to [`Provider<T>`](https://docs.rs/lockjaw/latest/lockjaw/struct.Provider.html), which creates a new
instance of `T`
everytime [`get()`](https://docs.rs/lockjaw/latest/lockjaw/struct.Provider.html#method.get)  is
called.

Since a `Provider` needs to use the component to create the instance, its lifetime is bound by the
component.

## Creating multiple instances

`Provider<T>` can be used to create instances on request.

```rust,no_run,noplayground
struct Foo {
  bar_provider: Provider<Bar>,
  bars : Vec<Bar>,
}

impl Foo{
    #[inject]
    pub fn new(bar_provider: Provider<Bar>) -> Foo {
      bar_provider,
      bars: vec![bar_provider.get(), bar_provider.get(), bar_provider.get()],
    }
    
    pub fn add_more_bar(&mut self){
       self.bars.push(self.bar_provider.get())
    }
}
```

## Bypassing runtime cyclic dependency

Since regular dependencies must be created before instantiating an object, cyclic dependencies will
result in a recursive stack overflow when the constructor is called. Lockjaw will detect this
situation and refuse to compile your project.

However sometimes the dependency is only used at runtime, not at object construction. This is
especially common when singleton classes need to refer to each other. By using
`Provider<T>` the cycle can be broken.

```rust,no_run,noplayground
{{#include ../../integration_tests/tests/provider_cyclic.rs:cyclic}}
```

In this example, while instantiating `Bar`, instantiates a `Foo`, `Bar` won't be created
until `Foo.create_bar()` is called, hence creating either won't trigger a stack overflow.

Trying to call `Provider.get` may still lead to a stack overflow, and Lockjaw cannot check this for
you.