Annotates a struct to declare a binding qualifier.

This allows the same type to be provided multiple times under different names. When providing/
requesting bindings a [`#[qualified]`](injectable#qualified) attribute can be added to refer to a
specific binding.

While the [new type idiom](https://doc.rust-lang.org/rust-by-example/generics/new_types.html) may be
preferred for such uses, qualifers are required for [`#[into_vec]`](module_attributes::into_vec)/
[`#[into_map]`](module_attributes::into_map) since lockjaw must be able to tell the binding is a
`Vec` or `HashMap`.

```
# use lockjaw::*;

#[qualifier]
pub struct Foo;

#[qualifier]
pub struct Bar;

pub struct MyModule {}

#[module]
impl MyModule {
    #[provides]
    #[qualified(Foo)]
    pub fn provide_foo_string() -> String {
        "foo".to_owned()
    }
    
    #[provides]
    #[qualified(Bar)]
    pub fn provide_bar_string() -> String {
        "bar".to_owned()
    }
    
    #[provides]
    pub fn provide_regular_string() -> String {
        "regular".to_owned()
    }
}

#[component(modules: [MyModule])]
pub trait MyComponent {

    #[qualified(Foo)]
    fn foo(&self) -> String;
    
    #[qualified(Bar)]
    fn bar(&self) -> String;
    
    fn regular(&self) -> String;
}

pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    assert_eq!(component.foo(), "foo");
    assert_eq!(component.bar(), "bar");
    assert_eq!(component.regular(), "regular");
}
epilogue!();
```