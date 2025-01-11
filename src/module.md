Annotates a impl block that defines the bindings.

To incorporate a module to the dependency graph, it should be included as a field in the
`modules` field in the [`#[component]`](component) annotation.

```
# use lockjaw::{epilogue, injectable,  component};
use lockjaw::{module};
pub struct FooModule {}

#[module]
impl FooModule {
    #[provides]
    pub fn provide_string() -> String {
        "foo".to_owned()
    }
}

#[component(modules : [FooModule])]
pub trait MyComponent {
    fn string(&self) -> String;
}

pub fn main() {
    let component = <dyn MyComponent>::new();
    assert_eq!(component.string(), "foo");
}
epilogue!();
```

If the module struct contains fields, it must use [`#[builder_modules]`](builder_modules) instead.

```
# use lockjaw::*;
# lockjaw::prologue!("src/lib.rs");
pub struct FooModule {
    value : String
}

#[module]
impl FooModule {
    #[provides]
    pub fn provide_string(&self) -> String {
        self.value.clone()
    }
}

#[builder_modules]
pub struct MyBuilderModules {
    foo : FooModule,
}

#[component(builder_modules : MyBuilderModules)]
pub trait MyComponent {
    fn string(&self) -> String;
}

pub fn main() {
    let component = <dyn MyComponent>::build(MyBuilderModules {
        foo : FooModule {
            value:"bar".to_owned()
        }
    });
    assert_eq!(component.string(), "bar");
}
epilogue!();
```

# Metadata

Module additional metadata in the form of
`#[module(key=value, key2=value)]`.

## `subcomponents`

**Optional** path or array of paths to [`#[subcomponent]`](subcomponent) the module should bind. The
subcomponent's builder will be bound with the module, and the subcomponent will have access to all
the bindings of the component/subcomponent the module is installed in.

## `install_in`

**Optional** path to a [`#[define_component]`](define_component)
/[`#[define_subcomponent]`](define_subcomponent) where the module will be automatically installed
in, instead of having to specify the module in a component's [`modules`](component#modules)
metadata. This allows a module to add bindings to a component that is defined in another crate the
current crate is depending on, For example injecting hooks into a library that will call it.

`install_in` is not allowed on modules with fields, as the component can't understand how to create
the module automatically.

# Method attributes

Methods in a module must have one of the [binding type](#binding-types) attribute. It may also have
additional [binding modifiers](#binding-modifiers)attributes that affects the behavior of the
binding.

Method attributes are nested under `#[module]`, and all nested attributes should be unqualified (
always used as `#[attribute]` instead of `#[lockjaw::attribute]`).

## Binding types

* [`#[provides]`](module_attributes::provides)
* [`#[binds]`](module_attributes::binds)
* [`#[binds_option_of]`](module_attributes::binds_option_of)
* [`#[multibinds]`](module_attributes::multibinds)

## Binding modifiers

* [`#[into_vec]`](module_attributes::into_vec)
* [`#[elements_into_vec]`](module_attributes::elements_into_vec)
* [`#[into_map]`](module_attributes::into_map)
* [`#[qualified]`](module_attributes::qualified)