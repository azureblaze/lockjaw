# Injecting Objects

## Declaring object injections

Lockjaw will create objects for you and satisfy their dependencies, but you need to let lockjaw know
how to create an object and what their dependencies are.

## Constructor injection

Lockjaw can create structs by calling a static method in the struct marked with the `#[inject]`
attribute. Lockjaw will satisfy the parameters with other injections. The impl block must be
annotated with `#[injectable]`, as Rust proc_macro cannot be applied to methods.

```rust
pub struct Foo{
    bar : Bar,
    i : i32
}

#[injectable]
impl Foo {
    #[inject]
    pub fn new(bar : crate::Bar) -> Foo {
        Foo {
            bar,
            i : 123
        }
    } 
}
```