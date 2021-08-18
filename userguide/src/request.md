# Requesting objects

`#[injectable]` and `#[provides]` defines how objects can be created, but they won't be too useful
unless someone tries to actually create an object.

Injected objects are created through a `#[component]`, which annotates a trait with methods that
return the requested types.

```rust
pub struct Foo {}

#[injectable]
impl Foo {
    #[inject]
    pub fn new() -> Self {
        Self {}
    }
}

#[component]
pub trait MyComponent {
    fn foo(&self) -> crate::Foo();
}
```

The methods should always take `&self`.

lockjaw will generate the implementation, including a static `new()` method that can be called to
create the component(as an opaque trait, box it if you want), which can be used to request the
objects.

```rust
fn main() {
    let my_component = MyComponent::new();
    let foo = my_component.foo();
}

epilogue!();
```

Note that if you forgot to call `epilogue!()`, or if there are any dependency issues, `new()` won't
be generated.