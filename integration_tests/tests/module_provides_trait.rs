#![allow(dead_code)]

use lockjaw::{component, epilogue, module};

lockjaw::prologue!("tests/module_provides_trait.rs");

pub struct Foo {}

pub trait MyTrait {
    fn hello(&self) -> String;
}

pub struct MyTraitImpl {}

impl MyTrait for MyTraitImpl {
    fn hello(&self) -> String {
        "hello".to_owned()
    }
}

pub struct MyModule {}
#[module]
impl MyModule {
    #[provides]
    pub fn provide_my_trait() -> Box<dyn crate::MyTrait> {
        Box::new(MyTraitImpl {})
    }
}

#[component(modules: MyModule)]
pub trait MyComponent {
    fn my_trait(&'_ self) -> Box<dyn crate::MyTrait + '_>;
}

#[test]
pub fn main() {
    lockjaw_init();
    let component: Box<dyn MyComponent> = <dyn MyComponent>::new();
    assert_eq!(component.my_trait().hello(), "hello");
}
epilogue!();
