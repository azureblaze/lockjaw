# Caveats

This section discusses the *creative solutions* Lockjaw used to achieve its goal. They are abhorrent
engineering practices that abuses undocumented behaviors of Rust, and are the main reasons you
should not use Lockjaw in any serious project.

They are documented so maybe someone can come up with a better solution, or Rust can provide new
language features to make Lockjaw usable.

## [Path resolution](path_resolution.md)

Lockjaw need to know the fully qualified path of a type, so they can be compared against each other.

## [Bypassing visibility](visibility_bypass.md)

A lot of symbols need should be private to the module/crate, but also give an exclusive bypass to
Lockjaw, so it can be used by a component generated elsewhere, possibly a different crate.

## [Late implementation generation](late_impl_generation.md)

Rust only allows `impl` blocks in the same `mod` the `struct` is in. However, some implementations
have to be generated at the `mod` root or a different crate, where information are more complete.