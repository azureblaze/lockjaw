Designates a [qualifier](crate::qualifier) to the return type, so a seperated binding of the same
type can be requested.

```
# use lockjaw::*;
# lockjaw::prologue!("src/lib.rs");

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