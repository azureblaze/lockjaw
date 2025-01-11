Represents "any component" which can be used with the `scope` and `install_in` metadata.

`Singleton` cannot be used with [`#[entry_point]`](entry_point). An entry point must be installed in
a specific component

```
# use lockjaw::*;
pub struct MyModule {}

struct Foo;

#[injectable(scope: Singleton)]
impl Foo {
    #[inject]
    pub fn new()-> Foo{
        Foo{}
    }
}

#[module(install_in: Singleton)]
impl MyModule {
    #[provides]
    pub fn provide_string() -> String {
        "string".to_owned()
    }
}

#[define_component]
pub trait MyComponent {
    fn string(&self) -> String;
}

pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    assert_eq!(component.string(), "string");
}
epilogue!();
```