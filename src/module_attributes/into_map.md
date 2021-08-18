Denotes the return value of the binding should be collected into a `HashMap<K,V>`.

`HashMap<K,V>` can then be depended on to access all bindings.

The value type of the map is determined by the return type. The key type is determined by additional
metadata on the attribute in the form of `#[into_map(metadata_key: metadata_value)]`.

Keys must be compile time constant.

# Metadata key `string_key`

The map type is be `HashMap<String, V>`. The metadata should have a string value which will be used
as the key for the binding.

# Metadata key `i32_key`

The map type is be `HashMap<i32, V>`. The metadata should have a `i32` integer value which will be
used as the key for the binding.

# Metadata key `enum_key`

The map type is be `HashMap<E, V>` where `E` is the type of the enum. The metadata be a path to the
enum value which wil be used as the key for the binding. The enum must be a simple enum (with no
structs, etc.), and must implement `Eq` and `Hash`

```
# use lockjaw::*;
# use std::collections::HashMap;
# lockjaw::prologue!("src/lib.rs");

#[derive(Eq, PartialEq, Hash)]
pub enum E {
    Foo,
    Bar,
}

pub struct MyModule {}

#[module]
impl MyModule {

    #[provides]
    #[into_map(string_key: "1")]
    pub fn provide_string1() -> String {
        "string1".to_owned()
    }

    #[provides]
    #[into_map(string_key: "2")]
    pub fn provide_string2() -> String {
        "string2".to_owned()
    }

    #[provides]
    #[into_map(i32_key: 1)]
    pub fn provide_i32_string1() -> String {
        "i32_string1".to_owned()
    }

    #[provides]
    #[into_map(i32_key: 2)]
    pub fn provide_i32_string2() -> String {
        "i32_string2".to_owned()
    }

    #[provides]
    #[into_map(enum_key: E::Foo)]
    pub fn provide_enum_string1() -> String {
        "Foo".to_owned()
    }

    #[provides]
    #[into_map(enum_key: E::Bar)]
    pub fn provide_enum_string2() -> String {
        "Bar".to_owned()
    }
}

#[component(modules: [MyModule])]
pub trait MyComponent {
    fn string_map(&self) -> std::collections::HashMap<String, String>;
    fn i32_map(&self) -> std::collections::HashMap<i32, String>;
    fn enum_map(&self) -> std::collections::HashMap<E, String>;
}

pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    
    let string_map = component.string_map();
    assert_eq!(string_map.get("1").unwrap(), "string1");
    assert_eq!(string_map.get("2").unwrap(), "string2");
    
    let i32_map = component.i32_map();
    assert_eq!(i32_map.get(&1).unwrap(), "i32_string1");
    assert_eq!(i32_map.get(&2).unwrap(), "i32_string2");
    
    let enum_map = component.enum_map();
    assert_eq!(enum_map.get(&E::Foo).unwrap(), "Foo");
    assert_eq!(enum_map.get(&E::Bar).unwrap(), "Bar");
}

epilogue!();
```