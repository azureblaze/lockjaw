Denotes the method as "injection constructor", which is the method lockjaw will call to create the
object.

The method must be static, and must return an instance of the struct.

The method can request other injectable objects with its parameters. Lockjaw will fulfil those
objects before calling the injection constructor.

# Parameter attributes

Additional attributes can be added to the parameter to affect how the method behaves.

Parameter attributes are added before the parameter name, for example

```ignore
pub fn foo(#[attribute] param1: ParamType)
```

## `#[qualified]`

Designates a [qualifier](crate::qualifier) to the parameter type, so a seperated binding of the same
type can be requested.