# Scoped Bindings

By default everytime a dependency needs to be satisfied, lockjaw creates a new instance, and move it
to the dependency (field or method parameter). This is not always desired since an object may be
used to carry some common state, and we want every type that depends on it to get a reference to a
single instance instead (singletons).

To do this, the `scope` metadata can be specified on a `#[injecatable]`, `#[provides]`, or
`#[binds]`, passing a component's fully qualified path as a string literal. This means there are
only one instance of the type for objects created by the same instance of component (they are not
global singletons, you can still have multiple instances if you have multiple components).

Other types can depend on a scoped type as a reference (`&T`) or `Cl<T>`

```rust
struct Foo {}

#[injectable(scope="crate::MyComponent")]
impl Foo {
    #[inject]
    pub fn new() -> Self {
        Self {}
    }
}

struct Bar<'a> {
    foo: &'a Foo
}

#[injectable]
impl Bar<'_> {
    #[inject]
    pub fn new(foo: &'_ Foo) -> Bar<'_> {
        Bar { foo }
    }
}

#[component]
trait MyComponent {
    fn bar(&'_ self) -> Bar<'_>;
}
```

`Cl<T>` allows a type to decouple itself from whether the type depended on is scoped or not. It may
be an owned instance or a shared instance, but the type does not care as it will not try to move it.

Scoped types or types that depends on them will have the same lifetime as the component.

Note that in most uses a scoped type probably should be mutable to make it useful. However we cannot
request it as `&mut T` since certainly multiple objects will try to request it. Scoped types must
implement [interior mutability](https://doc.rust-lang.org/reference/interior-mutability.html) itself
and use an immutable interface.

While lockjaw implementing an automatic `RefCell<T>` might make the user's life easier, it does not
suit multi-threaded use cases well. It is better that the user make careful consideration of its
mutability and threading story.