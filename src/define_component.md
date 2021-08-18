Generates a [`#[component]`](component) at the root of the binary.

If a component is annotated with `#[define_component]` instead of the regular `#[component]`
attribute, instead of having to list all [`#[module]`](module) lockjaw will gather every `#[module]`
with the [`install_in` metadata](module#install_in) from the binary's dependencies, and
automatically install them in the component.

All other operations are identical to a regular [`#[component]`](component), and the
[`modules` metadata](component#modules) can still be used.

The main advantage of using `#[define_component]` is inverting the dependency between a client and a
framework. With regular `#[component]` a framework must depend on the client since it has to
reference the client module for extensions, and the implementation has to be generated when the
framework is compiled. When `#[define_component]` is used, the framework merely defines the
component and adds its bindings to it, while the client will complete the dependency graph and the
implementation is automatically generated.

For subcomponents use [`#[define_sumcomponent]`](define_subcomponent) instead.

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

# Root crate

Using `#[define_component]`/`#[define_subcomponent]` requires a root crate, which is done by passing
`root` to the [`epilogue!()` macro](epilogue). Typically this is done on a binary.

A root crate generates the implementation for the component, hence preventing any further extensions
to the dependency graph. Lockjaw will prevent any other crates using lockjaw from depending directly
or indirectly on a root crate.

# Entry points

A regular `#[component]` requires all external access to the component to be done through the
component trait, which means the component user has to indirectly depend on all types the component
provides

When `#[define_component]`, lockjaw can also provide access to a subset of the dependency graph by
using [`#[entry_point]`](entry_point). `#[entry_point]` can take an opaque component trait and
safely access bindings from it(with compile time check). Users of the component can only require
bindings they need directly, without having to know about other things the component provides.