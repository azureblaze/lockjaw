# Lockjaw

Lockjaw is a fully static, compile-time [dependency injection](https://en.wikipedia.org/wiki/Dependency_injection) framework for [Rust](https://www.rust-lang.org/) inspired by [Dagger](https://dagger.dev).
It is also what you get when jabbed by a rusty dagger.

```rust
use lockjaw::{module, module_impl,component, injectable, root_epilogue, component_module_manifest};

#[injectable]
struct GreetCounter{
    counter : i32
}

impl GreetCounter{
    pub fn increment(&mut self) -> i32 {
        self.counter = self.counter + 1;
        self.counter
    }
}

pub trait Greeter {
    fn greet(&mut self) -> String;
}

#[injectable]
struct GreeterImpl {
    #[inject]
    greet_counter : crate::GreetCounter,
    #[inject]
    phrase : String
}

impl Greeter for GreeterImpl{
    fn greet(&mut self) -> String{
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
    fn greeter(&'_ self) -> Box<dyn crate::Greeter + '_>;
}

pub fn main() {
    let component: Box<dyn MyComponent> = MyComponent::new();
    let mut greeter = component.greeter();
    assert_eq!(greeter.greet(), "helloworld 1");
    assert_eq!(greeter.greet(), "helloworld 2");

    assert_eq!(component.greeter().greet(), "helloworld 1");
}

root_epilogue!("main.rs");
```

# Disclaimer

This is not an officially supported Google product

