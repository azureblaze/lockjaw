Annotates a trait that composes the dependency graph and provides items in the graph
(An "injector").

```rust
# #[macro_use] extern crate lockjaw_processor;
# #[injectable]
# struct Foo{}
#
#[component]
trait MyComponent {
    fn foo(&self) -> crate::Foo;
}

pub fn main() {
    let component: Box<dyn MyComponent> = MyComponent::new();
    let foo : Foo = component.foo();
}
# test_epilogue!();
```

# Creating injected types

A component can declare methods to allow injected types to be created for code outside the
dependency graph. the method should take only `&self` as parameter, and return the injected
type.

Methods in a component must take only `&self` as parameter, and return a injected type. If the
returned type is not injected compilation will fail.

See [`injectable`] and [`module`] for how to make a type injectable.

Most types used by lockjaw must be fully qualified, i.e. it must start with either `::` or
`crate::`. The only expections are types included in the rust [prelude](std::prelude):
*   [Box]
*   [Option]
*   [Result]
*   [String]
*   [Vec]

lockjaw will complain non-fully qualified type at compile time

```rust,compile_fail
# #[macro_use] extern crate lockjaw_processor;
# #[injectable]
# struct Foo{}
#[component]
trait MyComponent {
    fn foo(&self) -> Foo;
}

# fn main(){}
# test_epilogue!();
```
# Creating component instances

Lockjaw generates a `<COMPONENT>Builder` struct for every component, with a `new()` method
that returns a `Box<dyn COMPONENT>`.

```rust
# #[macro_use] extern crate lockjaw_processor;
#
#[component]
trait MyComponent {
    //...
}

pub fn main() {
    let component: Box<dyn MyComponent> = MyComponent::new();
}
# test_epilogue!();
```

Each instance of the component will have independent set of [scoped injections](docs::scoped)

# Installing modules
Each component can install their separate set of [`modules`](module) to form a different
dependency graph. Modules can be specified as comma separated string and set to the `modules`
argument.

```rust
# #[macro_use] extern crate lockjaw_processor;
# #[module]
# struct StringModule {}
# #[module_impl]
# impl StringModule {
#     #[provides]
#     pub fn provide_string() -> String {
#         "string".to_owned()
#     }
# }
#
# #[module]
# struct UnsignedModule {}
# #[module_impl]
# impl UnsignedModule {
#     #[provides]
#     pub fn provide_unsigned() -> u32 {
#         42
#     }
# }
#
#[component(modules = "crate::StringModule, crate::UnsignedModule")]
trait MyComponent {
    fn string(&self) -> String;
    fn unsigned(&self) -> u32;
}

# fn main() {}
# test_epilogue!();
```

Component can select different modules providing the same type to change the behavior of types
that depend on it.

```rust
# #[macro_use] extern crate lockjaw_processor;

#[injectable]
struct Foo{
    #[inject]
    string: String
}

# #[module]
# struct MyModule {}
#[module_impl]
impl MyModule {
    #[provides]
    pub fn provide_string() -> String { "string".to_owned() }
}

# #[module]
# struct OtherModule {}
#[module_impl]
impl OtherModule {
    #[provides]
    pub fn provide_string() -> String {"other_string".to_owned() }
}

#[component(modules = "crate::MyModule")]
trait MyComponent {
    fn foo(&self) -> crate::Foo;
}

#[component(modules = "crate::OtherModule")]
trait OtherComponent {
    fn foo(&self) -> crate::Foo;
}

pub fn main() {
    let component: Box<dyn MyComponent> = MyComponent::new();
    assert_eq!(component.foo().string, "string");
    let other_component: Box<dyn OtherComponent> = OtherComponent::new();
    assert_eq!(other_component.foo().string, "other_string");
}
# test_epilogue!();
```