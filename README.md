# Lockjaw

Lockjaw is a fully static, compile-time
[dependency injection](https://en.wikipedia.org/wiki/Dependency_injection) framework for
[Rust](https://www.rust-lang.org/) inspired by [Dagger](https://dagger.dev). It is also what you get
when jabbed by a rusty dagger.

Features:

* Compile time dependency graph validation with helpful diagnostic messages. The code won't compile
  if a dependency is missing or there are cyclic dependencies.
* Cross-crate injection. Libraries can provide different implementations like prod and test, and
  allow clients to choose from them.
* Aims for feature parity with Dagger. If you have used Dagger before, lockjaw should feel familiar.
    * Implemented:
        * [`@Inject`](https://docs.oracle.com/javaee/7/api/javax/inject/package-summary.html) =>
          [`#[inject]`](https://docs.rs/lockjaw/latest/lockjaw/injectable_attributes/attr.inject.html)
          constructor injection
          in [`#[injectable]`](https://docs.rs/lockjaw/latest/lockjaw/attr.injectable.html)
        * [`@Provides`](https://dagger.dev/api/latest/dagger/Provides.html)
          => [`#[provides]`](https://docs.rs/lockjaw/latest/lockjaw/module_attributes/attr.provides.html)
          bind method return values
        * [`@Binds`](https://dagger.dev/api/latest/dagger/Binds.html)
          => [`#[binds]`](https://docs.rs/lockjaw/latest/lockjaw/module_attributes/attr.binds.html)
          bind trait to implementation.
        * [`@Singleton`](https://docs.oracle.com/javaee/7/api/javax/inject/Singleton.html) /
          [`@Scope`](https://docs.oracle.com/javaee/6/api/javax/inject/Scope.html) =>
          [`scope=component`]((https://docs.rs/lockjaw/latest/lockjaw/attr.injectable.html#scope))
          shared instance.
        * [`@Named`](https://docs.oracle.com/javaee/6/api/javax/inject/Named.html) =>
          [`#[qualified]`](https://docs.rs/lockjaw/latest/lockjaw/module_attributes/attr.qualified.html)
        * [`Provider<T>`](https://docs.oracle.com/javaee/7/api/javax/inject/Provider.html)
          => [`Provider<T>`](https://docs.rs/lockjaw/latest/lockjaw/struct.Provider.html) create
          multiple instances at run time.
        * [`Lazy<T>`](https://dagger.dev/api/latest/dagger/Lazy.html)
          => [`Lazy<T>`](https://docs.rs/lockjaw/latest/lockjaw/struct.Lazy.html) create and cache
          instance only when used.
        * [Subcomponents](https://dagger.dev/dev-guide/subcomponents)
          => [`#[subcomponent]`](https://docs.rs/lockjaw/latest/lockjaw/attr.define_component.html)
          Dynamically creatable sub-scopes with additional bindings
        * [Multibindings](https://dagger.dev/dev-guide/multibindings)
          => [`#[into_vec]`](https://docs.rs/lockjaw/latest/lockjaw/module_attributes/attr.into_vec.html)
          / [`#[into_map]`](https://docs.rs/lockjaw/latest/lockjaw/module_attributes/attr.into_map.html)
          Collect same bindings to a Vec/HashMap, useful for plugin systems.
        * [`@BindsOptionalOf`](https://dagger.dev/api/2.13/index.html?dagger/BindsOptionalOf.html)
          => [`#[binds_option_of]`](https://docs.rs/lockjaw/latest/lockjaw/module_attributes/attr.binds_option_of.html)
          Allow some bindings to be missing
        * [Factories](https://github.com/google/auto/tree/master/factory)
          => [`#[facotry]`](https://docs.rs/lockjaw/latest/lockjaw/injectable_attributes/attr.factory.html)
          create objects with both injected fields and runtime fields.
        * [Hilt](https://dagger.dev/hilt/)
          => [`#[define_component]`](https://docs.rs/lockjaw/latest/lockjaw/attr.define_component.html)/ [`#[entry_point`](https://docs.rs/lockjaw/latest/lockjaw/attr.entry_point.html)
          / [`install_in`](https://docs.rs/lockjaw/latest/lockjaw/attr.module.html#install_in)
          Automatic module collection from build dependency.
    * Todo
        * [Producers](https://dagger.dev/dev-guide/producers) async dependency injection. Might not
          be too useful comparing to async/await

See [user guide](https://azureblaze.github.io/lockjaw/) for more information.

Example:

```rust
use lockjaw::*;
use std::ops::Add;

lockjaw::prologue!("src/lib.rs");

struct GreetCounter {
    counter: ::std::cell::RefCell<i32>
}

// Allow GreetCounter to be created in the dependency graph. These bindings are available anywhere.
#[injectable]
impl GreetCounter {
    // Marks a method as the inject constructor. Lockjaw will call this to create the object.
    #[inject]
    pub fn new() -> Self {
        Self{counter : std::cell::RefCell::new(0) }
    }
    
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

struct GreeterImpl {
    greet_counter : crate::GreetCounter,
    phrase : String
}

#[injectable]
impl GreeterImpl {
    // Lockjaw will call this with other injectable objects provided.
    #[inject]
    pub fn new(greet_counter : GreetCounter, phrase : String) -> Self {
        Self {
            greet_counter,
            phrase
        }
    }
}

impl Greeter for GreeterImpl{
    fn greet(&self) -> String{
        format!("{} {}", self.phrase, self.greet_counter.increment())
    }
}

// Declare a module so we can do special bindings. These bindings are only available if the
// component installs the module, so different bindings can be used based on the situation.
struct MyModule {}
#[module]
impl MyModule {
    // When ever someone needs a Greeter, use GreeterImpl as the actual implementation 
    #[binds]
    pub fn bind_greeter(_impl : crate::GreeterImpl) -> Cl<dyn Greeter> {}

    // Called when a String is requested
    #[provides]
    pub fn provide_string() -> String {
        "helloworld".to_owned()
    }
}

// Components stitch modules and injectables together into a dependency graph, and can create
// objects in the graph. The component installs modules listed in `modules`
#[component(modules: MyModule)]
trait MyComponent {
    // Allows creating a greeter with the component. The created object has the lifetime of the
    // component
    fn greeter(&self) -> Cl<dyn Greeter>;
}

pub fn main() {
    // Creates the component
    let component = MyComponent::new();
    // Creates a greeter.
    let greeter = component.greeter();
    assert_eq!(greeter.greet(), "helloworld 1");
    // Internal states of the greeter is kept.
    assert_eq!(greeter.greet(), "helloworld 2");
    
    // A new greeter has a new independent set of injected objects.
    assert_eq!(component.greeter().greet(), "helloworld 1");
}
// called after the last use of lockjaw to perform validation and code generation
epilogue!();
```

# Disclaimer

This is not an officially supported Google product.

Lockjaw is currently in early development and all APIs are subjected to changes. Some feature are
also implemented in a [hacky way](https://azureblaze.github.io/lockjaw/caveats.html). Use at your
own risk.