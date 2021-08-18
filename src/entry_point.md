Annotates a trait to provide bindings access to an opaque [`#[define_component]`](define_component)
/[`#[define_subcomponent]`](define_subcomponent) component.

```
# use lockjaw::*;
# prologue!("src/lib.rs");

// The component can be defined by other crates.
#[define_component]
trait MyComponent{}

struct MyModule {}

#[module(install_in: MyComponent)]
impl MyModule {
    #[provides]
    pub fn provide_i(&self) -> i32 {
        42
    }
}

#[entry_point(install_in: MyComponent)]
pub trait MyEntryPoint {
    fn i(&self) -> i32;
}

pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();

    assert_eq!(<dyn MyEntryPoint>::get(component.as_ref()).i(), 42)
}

lockjaw::epilogue!(root);
```

# Entry point methods

Entry point methods behaves the same as [component methods](component#component-methods).

# Entry point retriever

# Component builder

For a trait `FooEntryPoint` annotated with `#[entry_point(install_in: FooComponent)]`, a retriever
method is generated:

```ignore
impl Foo {
    pub fn get(component: &dyn FooComponent) -> &dyn FooEntryPoint
}
```

which can be used to cast the component to the entry point. Lockjaw checks at compile time the cast
is safe and the requests from the entry point can be fulfilled.

# Metadata

Entry points accept additional metadata in the form of `#[entry_point(key=value, key2=value2)]`.

## `install_in`

**Required** path to a [`#[define_component]`](define_component)
/[`#[define_subcomponent]`](define_subcomponent) trait