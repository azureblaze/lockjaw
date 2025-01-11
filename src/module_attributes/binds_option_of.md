Declares an optional binding.

If `#[binds_option_of] pub fn option_foo()->Option<Foo>` is declared, injecting `Option<Foo>` will
result in `Some(Foo)` if `Foo` is bound elsewhere. Otherwise, it results in `None`.

Typically, this is used if an optional feature is provided by another module which may not be
included in the component.