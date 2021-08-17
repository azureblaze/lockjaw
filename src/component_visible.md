Annotates a non-public [`[injectable]`](injectable) struct, a [`#[module]`](module) struct, or a
trait used by components so their implementation can be generated.

Most lockjaw code generation is done at the crate root or even in a different crate, and the item it
uses may not always be visible at the generation site. Hence trying to make a private struct
injectable may result in
a [private type in public interface](https://doc.rust-lang.org/error-index.html#E0446) error, even
though the struct is only injected at places that has visibility.

This annotation exposes a lockjaw-only visibility so code generation can use them.

IMPORTANT: The `#[component_visible]` annotation does not work on mods. All mods that contains
bindings must be at least visible to the crate root.

WARNING: using this annotation on public items may cause mangled item appearing in rustdoc.

# Implementation notes

Under the hood this annotation renames the item declaration to an internal name which is made
public, and then `use` the internal name as the original name as the original visibility. Hence,
when trying to access the item with the original name will be restricted to the original visibility,
while the internal name can be publicly used by lockjaw.
