# Lockjaw

Lockjaw is a fully static, compile-time [dependency injection](https://en.wikipedia.org/wiki/Dependency_injection) framework for [Rust](https://www.rust-lang.org/) inspired by [Dagger](https://dagger.dev).
It is also what you get when jabbed by a rusty dagger.

```rust
use lockjaw::{module, module_impl,component, injectable, test_epilogue, component_module_manifest, MaybeScoped};
use std::ops::Add;
#[injectable]
struct GreetCounter{
    counter : ::std::cell::RefCell<i32>
}

impl GreetCounter{
    pub fn increment(&self) -> i32 {
        let mut m = self.counter.borrow_mut();
        *m = m.add(1);
        m.clone()
    }
}

pub trait Greeter {
    fn greet(&self) -> String;
}

#[injectable]
struct GreeterImpl {
    #[inject]
    greet_counter : crate::GreetCounter,
    #[inject]
    phrase : String
}

impl Greeter for GreeterImpl{
    fn greet(&self) -> String{
        format!("{} {}", self.phrase, self.greet_counter.increment())
    }
}

#[module]
struct MyModule {}

#[module_impl]
impl MyModule {
    #[binds]
    pub fn bind_greeter(_impl : crate::GreeterImpl) -> impl crate::Greeter {}

    #[provides]
    pub fn provide_string() -> String {
        "helloworld".to_owned()
    }
}

#[component_module_manifest]
struct ModuleManifest (crate::MyModule);

#[component(modules = "crate::ModuleManifest")]
trait MyComponent {
    fn greeter(&'_ self) -> MaybeScoped<'_, dyn crate::Greeter + '_>;
}

pub fn main() {
    let component: Box<dyn MyComponent> = MyComponent::new();
    let greeter = component.greeter();
    assert_eq!(greeter.greet(), "helloworld 1");
    assert_eq!(greeter.greet(), "helloworld 2");

    assert_eq!(component.greeter().greet(), "helloworld 1");
}

test_epilogue!();
```

# Disclaimer

This is not an officially supported Google product

