# Lockjaw

Lockjaw is a fully static, compile-time
[dependency injection](https://en.wikipedia.org/wiki/Dependency_injection) framework for
[Rust](https://www.rust-lang.org/) inspired by [Dagger](https://dagger.dev).
It is also what you get when jabbed by a rusty dagger.

Features:
*   Compile time dependency graph validation with helpful  diagnostic messages. The code won't
    compile if a dependency is missing or there are cyclic dependencies.
*   Cross-crate injection. Libraries can provide different implementations like prod and test, and
    allow clients to choose from them.
*   Aims for feature parity with Dagger. If you have used Dagger before, lockjaw should feel
    familiar.
    *   Implemented:
        *   [`@Inject`](https://dagger.dev/members-injection.html) =>
            `#[injectable]` member injection for concrete class
        *   [`@Provides`](https://dagger.dev/api/latest/dagger/Provides.html) => `#[provides]` bind
            method return values
        *   [`@Binds`](https://dagger.dev/api/latest/dagger/Binds.html) => `#[binds]` bind trait to
            implementation.
        *   [`@Singleton`](https://docs.oracle.com/javaee/7/api/javax/inject/Singleton.html) /
            [`@Scope`](https://docs.oracle.com/javaee/6/api/javax/inject/Scope.html) =>
            `scope="component"` shared instance.
        *   [`@Named`](https://docs.oracle.com/javaee/6/api/javax/inject/Named.html) =>
            `type NamedType = Type;`
    *   To do:
        *   `@Inject` constructor injection
        *   [`Provider<T>`](https://docs.oracle.com/javaee/7/api/javax/inject/Provider.html) create
            multiple instances at run time.
        *   [`Lazy<T>`](https://dagger.dev/api/latest/dagger/Lazy.html) create and cache instance
            only when used.
        *   [Subcomponents](https://dagger.dev/dev-guide/subcomponents) Dynamically creatable
            sub-scopes with additional bindings
        *   [Multibindings](https://dagger.dev/dev-guide/multibindings) Collect same bindings to
            a set/map, useful for plugin systems.
        *   [`@BindsOptionalOf`](https://dagger.dev/api/2.13/index.html?dagger/BindsOptionalOf.html)
            Allow some bindings to be missing
        *   [Factories](https://github.com/google/auto/tree/master/factory) create objects with
            both injected fields and runtime fields.
        *   [Producers](https://dagger.dev/dev-guide/producers) async dependency injection. Might
            not be too useful comparing to async/await

## Using lockjaw

See also the [example project](https://github.com/azureblaze/lockjaw/tree/main/example).

### Setup
Add lockjaw to your `Cargo.toml`:
```
lockjaw = "*"
```

:warning: **Lockjaw is still in early development.** all APIs are subjected to change and will
break without notice. You may want to pin it to a specific version, but if you don't want to fix
breaking changes, you probably should not use lockjaw yet.

The proc_macro and runtime library are packaged into the same crate, so this is the only target
you need.

Lockjaw also needs some environment setup, and requires a
[build script](https://doc.rust-lang.org/cargo/reference/build-scripts.html). Add `build.rs` next to
`Cargo.toml`, and call `lockjaw::build_script()` in `main()` inside it:

```rust
// build.rs
fn main(){
    lockjaw::build_script();
}
```

Lockjaw will ask you to do this if this step is missing.

You also must call the `lockjaw::epilogue!()` macro in the root of your crate (`lib.rs` or
`main.rs`) after all other uses of lockjaw, preferably at the end of the file.

### Fully qualified types

Lockjaw and proc_macros operate at the token phase, and have no idea what types are available. It
cannot resolve `use` and type aliases. Hence, wherever lockjaw uses a type, it must be fully
qualified, meaning its [path](https://doc.rust-lang.org/reference/paths.html) must start with `::`
or `crate` for local types. Even if it is a type from `std` it still needs to be qualified as 
`::std::...`.

The only exceptions are types Rust `use` in the
[prelude](https://doc.rust-lang.org/std/prelude/index.html), including:
* [`Box<T>`](https://doc.rust-lang.org/std/boxed/struct.Box.html)
* [`Option<T>`](https://doc.rust-lang.org/std/option/enum.Option.html)
* [`Result<T, R>`](https://doc.rust-lang.org/std/result/enum.Result.html)
* [`String`](https://doc.rust-lang.org/std/string/struct.String.html)
* [`Vec<T>`](https://doc.rust-lang.org/std/vec/struct.Vec.html)
* Primitive types (`i32`, `bool`, etc. they don't belong to any namespace anyway.)

as they are expected to be commonly used. Internally they are automatically expanded to their full
name.

Lockjaw runtime library types also do not need to be qualified, which are:
* `ComponentLifetime<T>`

Type aliases are interpreted as completely different types, and will need their dependencies
satisfied separately.


### Declaring object injections

Lockjaw will create objects for you and satisfy their dependencies, but you need to let lockjaw know
how to create an object and what their dependencies are.

#### Constructor injection

Lockjaw can create structs by calling a static method in the struct marked with the `#[inject]`
attribute. Lockjaw will satisfy the parameters with other injections. The impl block must be
annotated with `#[injectable]`, as Rust proc_macro cannot be applied to methods.

```rust
pub struct Foo{
    bar : Bar,
    i : i32
}

#[injectable]
impl Foo {
    #[inject]
    pub fn new(bar : crate::Bar) -> Foo {
        Foo {
            bar,
            i : 123
        }
    } 
}
```

### More complicated object bindings

Injections does not always work:
*   Traits are not structs and can not be instantiated.
*   `#[injectable]` and `#[inject]` cannot annotate third party structs.
*   The desired implementation of a trait may be affected by run time conditions, like switching
    implementations based on reading a config file.
    
For these cases, a `#[provides]` method can be used instead. A `#[provides]` method satisfy its
return type. When the type is requested the method is invoked.

```rust
#[provides]
pub fn provide_string() -> String {
    "my_string".to_owned()
}
```

Whenever someone requested a `String`, they will get a `String` with the value `"my_string"`.

`#[provides]` method can also have their own dependencies, which is requested through the method
parameter. Lockjaw will satisfy them first and pass them to the method.

```rust
pub struct Foo {
    value: String
}

#[provides]
pub fn provide_foo(s : String) -> crate::Foo {
    Foo {value: s}
}
```

When someone requested a `Foo`, `provided_foo()` will be invoked with the `String` returned by 
`provided_string()`, and used to create the `Foo` struct.

`#[provides]` can also be used for traits:

```rust
pub trait Foo {
    fn foo();
}

pub struct FooImpl {}

#[injectable]
impl FooImpl {
    #[inject]
    pub fn new() -> Self {
        Self {}
    }
}

impl Foo for FooImpl {
    fn foo() {
        //...
    }
}

#[provides]
pub fn provide_foo(impl_ : FooImpl) -> Box<dyn Foo> {
    Box::new(impl_)
}
```

We can also simplify trait provisions with a `#[binds]` method:

```rust
#[binds]
pub fn bind_foo_impl(impl_ : FooImpl) -> impl Foo {}
```

Note that while `#[binds]` returns `impl Foo`, the actual binding is `ComponentLifetime<Foo>`,
since the implementation may contain a reference to a singleton (more on [scope](#scoped-bindings)
later). Currently `#[binds]` must return `impl T`, but in the future this will be changed into
`ComponentLifetime<T>` to make it more intuitive.

Since `#[provides]` and `#[binds]` are less coupled with the type they provide, and users might
want to swap out implementations (like use a `FakeClient` for test that emulates talking to a 
server without actual network operations), these bindings should not be global. They are grouped
into a `#[module]` so they can be incorporated later into a specific dependency graph. `#[module]`
annotates a [impl block](https://doc.rust-lang.org/std/keyword.impl.html) of a struct to define the
bindings.

```rust
struct MyModule {}
#[module]
impl MyModule {
    #[binds]
    pub fn bind_foo_impl(impl_ : FooImpl) -> impl Foo {}
}
```

Note: in lockjaw documentations, "modules" always refer to the dependency injection module. The Rust
module is always referred as `mod`.

### Requesting objects

`#[injectable]` and `#[provides]` defines how objects can be created, but they won't be too useful
unless someone tries to actually create an object.

Injected objects are created through a `#[component]`, which annotates a trait with methods that
return the requested types.

```rust
pub struct Foo {}

#[injectable]
impl Foo {
    #[inject]
    pub fn new() -> Self {
        Self {}
    }
}

#[component]
pub trait MyComponent {
    fn foo(&self) -> crate::Foo();
}
```

The methods should always take `&self`.

lockjaw will generate the implementation, including a static `new()` method that can be called to
create the component(as an opaque trait, box it if you want), which can be used to request the
objects.

```rust
fn main() {
    let my_component = MyComponent::new();
    let foo = my_component.foo();
}

epilogue!();
```

Note that if you forgot to call `epilogue!()`, or if there are any dependency issues, `new()` won't
be generated.

### Installing modules

`#[injectables]` are global bindings, which mean they are included into any dependency graph. But
for `#[provides]` which bindings should be available should be specified, as it may be situational.

Each component should declare a list of modules they want to use with a  
`#[component_module_manifest]` struct. The component should then use the `modules` metadata to
select the manifest it wants.

```rust
#[component_module_manifest]
struct MyModuleManifest {
    my_module: crate::MyModule
}

#[component(modules="crate::MyModuleManifest")]
trait MyComponent {
    fn my_module_binding() -> crate::MyModuleBinding;
}
```

### Scoped bindings

By default everytime a dependency needs to be satisfied, lockjaw creates a new instance, and move
it to the dependency (field or method parameter). This is not always desired since an object may
be used to carry some common state, and we want every type that depends on it to get a reference
to a single instance instead (singletons).

To do this, the `scope` metadata can be specified on a `#[injecatable]`, `#[provides]`, or 
`#[binds]`, passing a component's fully qualified path as a string literal. This means there
are only one instance of the type for objects created by the same instance of component (they
are not global singletons, you can still have multiple instances if you have multiple components).

Other types can depend on a scoped type as a reference (`&T`) or `ComponentLifetime<T>`

```rust
struct Foo {}

#[injectable(scope="crate::MyComponent")]
impl Foo {
    #[inject]
    pub fn new() -> Self {
        Self {}
    }
}

struct Bar<'a> {
    foo: &'a Foo
}

#[injectable]
impl Bar<'_> {
    #[inject]
    pub fn new(foo: &'_ Foo) -> Bar<'_> {
        Bar { foo }
    }
}

#[component]
trait MyComponent {
    fn bar(&'_ self) -> Bar<'_>;
}
```

`ComponentLifetime<T>` allows a type to decouple itself from whether the type depended on is scoped or
not. It may be an owned instance or a shared instance, but the type does not care as it will not
try to move it.

Scoped types or types that depends on them will have the same lifetime as the component.

Note that in most uses a scoped type probably should be mutable to make it useful. However we cannot
request it as `&mut T` since certainly multiple objects will try to request it. Scoped types must
implement [interior mutability](https://doc.rust-lang.org/reference/interior-mutability.html) itself
and use an immutable interface.

While lockjaw implementing an automatic `RefCell<T>` might make the user's life easier, it does not
suit multi-threaded use cases well. It is better that the user make careful consideration of its
mutability and threading story.

## [Caveats](caveats.md)
            
## [Code of conduct](code-of-conduct.md)

## [Contributing](contributing.md)