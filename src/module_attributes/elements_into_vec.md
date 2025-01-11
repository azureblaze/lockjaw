Similar to [`#[into_vec]`](into_vec) but instead of a single element, all elements in the returned
`Vec<T>` is merged into the `Vec<T>` binding.

This allows the module to inject multiple elements into the `Vec<T>`, or conditionally inject no
elements.

```
# use lockjaw::*;
struct MyModule;

#[module]
impl MyModule {

    #[provides]
    #[into_vec]
    pub fn provide_string1() -> String {
        "string1".to_owned()
    }

    #[provides]
    #[elements_into_vec]
    pub fn provide_string2() -> Vec<String> {
        vec!["string2".to_owned(), "string3".to_owned()]
    }
    
    #[provides]
    #[elements_into_vec]
    pub fn provide_string4() -> Vec<String> {
        if true {
            vec![]        
        } else {
            vec!["string4".to_owned()]
        }
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
    assert!(v.contains(&"string3".to_owned()));
    
    assert!(!v.contains(&"string4".to_owned()));
}

epilogue!();
```