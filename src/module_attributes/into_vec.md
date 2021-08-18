Denotes the return value of the binding should be collected into a `Vec<T>`.

`Vec<T>` can then be depended on to access all bindings of `T`.

A module provide the binding to the `Vec<T>` at most once. However if 2 different module provides a
binding with the same value it will not be deduplicated.

The counterpart of `#[into_vec]` in Dagger is `@IntoSet`. Since `eq`/`hash` is less universally
available in Rust `Vec<T>` is the chosen collection.

If a `#[into_vec]` binding is also [`#[qualified(Q)]`](qualified), the result is collected into
`#[qualified(Q)] Vec<T>`.

```
# use lockjaw::*;
# lockjaw::prologue!("src/lib.rs");
struct MyModule;

#[module]
impl MyModule {

    #[provides]
    #[into_vec]
    pub fn provide_string1() -> String {
        "string1".to_owned()
    }

    #[provides]
    #[into_vec]
    pub fn provide_string2() -> String {
        "string2".to_owned()
    }
}

#[component(modules: MyModule)]
pub trait MyComponent {
    fn vec_string(&self) -> Vec<String>;
}

pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    let v = component.vec_string();
    assert!(v.contains(&"string1".to_owned()));
    assert!(v.contains(&"string2".to_owned()));
}

epilogue!();
```
