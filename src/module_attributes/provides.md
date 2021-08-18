Annotates a method that provides an object into the dependency graph.

When an object of the return type is depended on, this method will be called to create the object.
Other dependencies can be requested with the method parameter. `&self` can also be used to access
runtime values stored in the module.

```
# use lockjaw::*;
# lockjaw::prologue!("src/lib.rs");
pub struct Bar {}
#[injectable]
impl Bar {
    #[inject]
    pub fn new()-> Self {
        Self {}
    }
}

impl Bar {
    pub fn get_string(&self) -> String {
        "bar".to_owned()
    }
}

pub struct FooModule {
    value : String
}

#[module]
impl FooModule {
    #[provides]
    pub fn provide_string(&self, bar : crate::Bar) -> String {
        format!("{} {}",self.value.clone(), bar.get_string() )
    }
}

#[builder_modules]
pub struct MyBuilderModules {
    foo : crate::FooModule,
}

#[component(builder_modules : crate::MyBuilderModules)]
pub trait MyComponent {
    fn string(&self) -> String;
}

pub fn main() {
    let component = MyComponent::build(MyBuilderModules {
        foo : FooModule {
            value:"foo".to_owned()
        }
    });
    assert_eq!(component.string(), "foo bar");
}
epilogue!();

```

Cannot annotate a method that is already annotated with [`#[binds]`](#binds)

#### Metadata

`#[provides]` accept additional metadata in the form of
`#[provides(key=value, key2=value)]`.

##### scope

**Optional** fully qualified path to a [`component`](crate::component), which makes the returned
object a scoped singleton under the `component`.

The return object will only be provided in the `component`, and all objects generated from the
same `component` instance will share the same scoped returned object. Since it is shared, the scoped
returned object can only be depended on as  `&T` or [`Cl<T>`](crate::Cl), and the scoped returned
object or any objects that depends on it will share the lifetime _of_ the
`component`.

```
# use lockjaw::*;
# lockjaw::prologue!("src/lib.rs");

pub struct Foo {}

pub struct FooModule {}

#[module]
impl FooModule {
    #[provides(scope : crate::MyComponent)]
    pub fn provide_foo() -> crate::Foo {
        Foo{}
    }
}

pub struct Bar<'a>{
    foo : &'a crate::Foo
}

#[injectable]
impl Bar<'_> {
    #[inject]
    pub fn new(foo : &'_ crate::Foo) -> Bar<'_> {
        Bar { foo }
    }
}

#[component(modules : FooModule)]
pub trait MyComponent {
    fn bar(&self) -> crate::Bar;
}

pub fn main() {
    let component = <dyn MyComponent>::new();
    let bar1 = component.bar();
    let bar2 = component.bar();
    let bar1_ptr: *const Bar = &bar1;
    let bar2_ptr: *const Bar = &bar2;
    assert_ne!(bar1_ptr, bar2_ptr);
    let foo1_ptr : *const Foo = bar1.foo;
    let foo2_ptr : *const Foo = bar2.foo;
    assert_eq!(foo1_ptr, foo2_ptr);
}
epilogue!();
```

Scoped returned objects are shared and cannot be mutable while they commonly needs mutability. users
must implement internal mutability.
