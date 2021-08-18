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