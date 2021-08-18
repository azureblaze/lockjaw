Annotates a trait that composes a sub-dependency graph, which has additional bindings/scoped
bindings, and can also access bindings from the parent [`#[component]`](crate::component)/
`#[subcomponent]`.

`#[subcomponent]` can be seen as some sort of "session". For example a web server can have a
`#[component]` for global helpers, and creates a `#[subcomponent]` for each request using the URL.
Bindings within the `#[subcomponent]` can depend on the URL and other bindings derived from it, or
have a scoped shared object for the same request, while being isolated from other requests.

Operations on a `#[subcomponent]` is almost identical to a `#[component]`(crate::component), other
than it's extended dependency graph and creation of the `#[subcomponent]`

A subcomponent cannot outlive its parent component.

```
# use lockjaw::*;
# prologue!("src/lib.rs");
struct SubcomponentModule {}

#[module]
impl SubcomponentModule {
    #[provides]
    pub fn provide_i32() -> i32 {
        32
    }
}

#[subcomponent(modules: [SubcomponentModule])]
pub trait MySubcomponent<'a> {
    fn fi64(&self) -> i64;
    fn fi32(&self) -> i32;
}

struct ParentComponentModule {}

#[module(subcomponents: [MySubcomponent])]
impl ParentComponentModule {
    #[provides]
    pub fn provide_i64() -> i64 {
        64
    }
}

#[component(modules: [ParentComponentModule])]
pub trait MyComponent {
    fn sub(&self) -> Cl<dyn MySubcomponentBuilder>;
}

pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    let sub: Cl<dyn MySubcomponent> = component.sub().build();
    // bindings in the subcomponent.
    assert_eq!(sub.fi32(), 32);
    // bindings in the parent component is still accessible.
    assert_eq!(sub.fi64(), 64);
}

lockjaw::epilogue!();
```

# Installing a subcomponent

A subcomponent can be installed to a parent component by specifying
the [`subcomponents` metadata](module#subcomponents) in a `#[module]`, and adding the module to the
parent component.

Once installed, the [subcomponent builder](#subcomponent-builder) binding will be provided to the
parent component, which can be used to create a subcomponent instance.

# Subcomponent builder

For a `Foo` subcomponent, instead of adding a static `build` method to the trait, lockjaw generates
a new builder trait in the same mod:

```ignore
[subcomponent visibility] trait FooBuilder<'parent_component> {
    fn build(&self, modules: BUILDER_MODULES) -> Box<dyn Foo>
}
```

If the `builder_modules` metadata is not provided, the `modules` parameter will be omitted, and the
signature becomes `pub fn build(&self) -> Box<dyn Foo>`

Calling `build()` creates a new instance of the subcomponent.

`FooBuilder` is injectable in components where it is [installed](#installing-a-subcomponent) in the
form of `Cl<dyn FooBuilder>`.

A subcomponent builder cannot outlive its parent component.

# Metadata

## `modules`

## `parent`

Path to a [`#[define_component]`](define_component)/[`#[define_subcomponent]`](define_subcomponent)
trait to specify the parent of this subcomponent. The subcomponent will have access to the parent's
bindings, and the subcomponent builder will be bound in the parent component.

If the parent is a [`#[component]`](component)/[`#[subcomponent]`](define_subcomponent), the
[`subcomponents` metadata in `#[module]`](module#subcomponents) should be used instead.

See [`modules` metadata in `#[component]`](component#modules)

## `builder_modules`

See [`builder_modules` metata in `#[component]`](component#builder_modules)

# Component methods

See [component methods in `#[component]`](component#component-methods)