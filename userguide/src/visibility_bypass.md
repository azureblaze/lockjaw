# Bypassing visibility

Visibility control works a bit weird with dependency injection. When a type is private, only context
that have visibility should be able to inject it, but the dependency injection framework should
still be able to construct it, even if the generated code is in some random `mod`. Currently Rust
only allow visibility bypass to be granted to a `mod` that is a parent of the current `mod`.

Lockjaw handles this with
the [`#[component_visible]`](https://docs.rs/lockjaw/0.2.1/lockjaw/attr.component_visible.html)
attribute macro. The macro modifies the `struct` declaration, so it is declared as public with a
hidden name, and then alias the original name with the original visibility. Internally Lockjaw uses
the hidden name. Everything is actually public.

This type of hack is hard to perform on a `mod`, so every `mod` that has a binding must be visible
to the crate root (using `pub(crate)`). Lockjaw then reexport the hidden type as public at the crate
root.