# Path resolution

Lockjaw need to know the fully qualified path of a type, so they can be compared against each other.

In Rust, all a `proc_macro` can see is tokens, which is too early to resolve the type path. When a
`Foo` identifier is encountered, it is difficult for the macro to understand whether it is a type
declared in the local module, or a type from somewhere else brought in by a `use` declaration. Rust
don't even tell the macro what the local module is.

## Base `mod` of the file

The first problem is the `proc_macro` doesn't even know where the source being compiled is at. The
[`file!()`](https://doc.rust-lang.org/std/macro.file.html)
and [`module_path!()`](https://doc.rust-lang.org/std/macro.module_path.html) would be a perfect
solution to this, but [eager macro expansion](https://github.com/rust-lang/rfcs/pull/2320) is
required for a `proc_macro` to be able to utilize it.

[proc_macro2::Span::source_file()](https://docs.rs/proc-macro2/1.0.28/proc_macro2/struct.Span.html#method.source_file)
also exists, but it is nightly feature and requires `procmacro2_semver_exempt` which is contagious.

In the Lockjaw build script this is resolved by looking at the crate's manifest and using
`CARGO_CRATE_NAME`/`CARGO_PKG_NAME` along with `cargo metadata` to figure out the exact path to the source. Path info
is only needed during dependency gathering which is no longer done with `proc_macro`, and component generation which is
always `::crate::` and never reference by other part of the code.

## `mod` structure and `use` declarations

A file can still contain nested `mod` in it, each importing more symbols with the `use` declaration.
For a given token, lockjaw needs to know which `mod` it is in, and what symbols are brought into
that scope. This requires parsing the whole file, so we can keep what the span of each `mod` is and
what `use` are inside it.

[`syn::parse_file()`](https://docs.rs/syn/1.0.75/syn/fn.parse_file.html) sounds like a good fit for
this, however the tokens it produces does not record
proper [spans](https://docs.rs/proc-macro2/1.0.28/proc_macro2/struct.Span.html), so we cannot use it
to find the position of `mod`.

Lockjaw handles this by parsing the whole file in the build script so it knows which `mod` it is in.
