Generate a factory that can construct the [`#[injectable]`](crate::injectable) with runtime
arguments in addition to injected fields.

Parameters in the method can be annotated with [`#[runtime]`](#runtime), which will be requested by
the factory during runtime when creating the struct. Unannotated parameters will be provided with
the dependency graph. The factory prepares the arguments, and calls the `#[factory]` method.

An `injectable` can only have one method annotated with either `#[inject]` or `#[factory]`. The
method must be static, and must return an instance of the struct.

Consider using [`Provider`](crate::Provider) instead if there are no runtime parameters, and
multiple instances of the struct needs to be created at runtime.

```
# use lockjaw::{epilogue, injectable, module, component};
# lockjaw::prologue!("src/lib.rs");

struct MyModule;

#[module]
impl MyModule {
    #[provides]
    pub fn provide_string(&self) -> String {
        "helloworld".to_owned()
    }
}

pub struct Foo {
    pub i: i32,
    pub phrase: String,
}

#[injectable]
impl Foo {
    #[factory]
    fn create(#[runtime] i: i32, phrase: String) -> Self {
        Self { i, phrase }
    }
}

#[component(modules: MyModule)]
pub trait MyComponent {
    fn foo_factory(&self) -> FooFactory;
}

pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::build();

    let foo = component.foo_factory().create(42);

    assert_eq!(foo.i, 42);
    assert_eq!(foo.phrase, "helloworld");
}

epilogue!();
```

# Parameter attributes

Additional attributes can be added to the parameter to affect how the method behaves.

Parameter attributes are added before the parameter name, for example:

```ignore
pub fn foo(#[attribute] param1: ParamType)
```

## `#[qualified]`

Designates a [qualifier](crate::qualifier) to the parameter type, so a seperated binding of the same
type can be requested.

## `#[runtime]`

Denotes the parameter must be passed by the caller when the factory method is called, instead of
being provided by the dependency graph.

The parameter will become a part of the generated factory method's parameter, in the same order they
are declared. Parameters without `#[runtime]` are stripped from the generated factory method.

### Generated code

For a struct `Foo` with a `[factory]` method called `create_foo`:

```ignore
struct Foo {...}

#[injectable]
impl Foo {
   #[factory]
   pub fn create_foo(#[runtime] runtime_1: Type1, injected: Injected) -> Foo {
      ...
   }
}
```

The struct `FooFactory<'component>` will be generated at the same module.

```ignore
struct FooFactory<'component> { ... }

impl FooFactory<'_> {
   pub fn create_foo(&self, runtime_1 : Type1) -> Foo {
      Foo::create_foo(runtime_1, self.injected.get())
   }
}
```

The factory contains a method with the same name as the `#[factory]` method. The factory can
implement a trait instead by using the [`implementing`](#implementing) metadata.

The factory depend on bindings from the component, hence cannot outlive it.

`FooFactory` has private visibility by default, which can be overridden by using the
[`visibility`](#visibility) metadata.

### Metadata

Injectable factories accept additional metadata in the form of
`#[factory(key=value, key2=value)]`.

#### `implementing`

**Optional** path to a trait which the factory will implement, instead of adding a method to the
factory struct.

For a struct `Foo` , the factory trait should have a method with the signature of
`fn create_foo(&self, runtime_parameters, ...) -> Foo`. The name of the `#[factory]` method should
match the trait method.

```
# use lockjaw::{epilogue, injectable, module, component, Cl};
# lockjaw::prologue!("src/lib.rs");
pub struct Foo {
    pub i: i32,
    pub phrase: String,
}

pub trait FooCreator {
  fn create_foo(&self, i: i32) -> Foo;
}

#[injectable]
impl Foo {
    #[factory(implementing: FooCreator)]
    fn create_foo(#[runtime] i: i32, phrase: String) -> Self {
        Self { i, phrase }
    }
}


struct MyModule;

#[module]
impl MyModule {
    #[provides]
    pub fn provide_string(&self) -> String {
        "helloworld".to_owned()
    }
    
    #[binds]
    pub fn bind_foo_creator(impl_: FooFactory) -> Cl<dyn FooCreator> {}
}

#[component(modules: MyModule)]
pub trait MyComponent {
    fn foo_creator(&self) -> Cl<dyn FooCreator>;
}

pub fn main() {
    let component: Box<dyn MyComponent> = <dyn MyComponent>::build();

    let foo = component.foo_creator().create_foo(42);

    assert_eq!(foo.i, 42);
    assert_eq!(foo.phrase, "helloworld");
}

epilogue!();
```

#### `visibility`

**Optional** string specifying the visibility of the generated factory. The string must conform to
the [rust visibility syntax](https://doc.rust-lang.org/reference/visibility-and-privacy.html), e.g.
`"pub"`, `"pub(crate)"`, `"pub(super)"`, or `"pub(in path::to::mod)"`

The factory is private by default.