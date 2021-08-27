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

Since the with [cross macro communication](cross_macro_communication.md) hacks the user only need to
do this once per file, we've decided to let the user pass the current path with
the [`prologue!()`](https://docs.rs/lockjaw/latest/lockjaw/macro.prologue.html) macro. We will need
to parse the whole file later anyway, so we take the file name and derive the `mod` path from it.

To make sure the `prologue!()` macro is called in every file, it declares a hidden symbol locally
which all other Lockjaw `proc_macro` will try to use, so if the `prolouge!()` is missing compilation
will fail. In later steps we also verify `prologue!()` is the first Lockjaw macro called in the
file, as the current file info is stored in global memory and must be reset in each file.

`prolouge!()` also generates a test to make sure the path passed in matches what `file!()` would
give. However using the wrong path will usually cause Lockjaw to fail miserably since all type info
are messed up, and the test will not even be run, which makes it not too useful.

## `mod` structure and `use` declarations

A file can still contain nested `mod` in it, each importing more symbols with the `use` declaration.
For a given token, lockjaw needs to know which `mod` it is in, and what symbols are brought into
that scope. This requires parsing the whole file, so we can keep what the span of each `mod` is and
what `use` are inside it.

[`syn::parse_file()`](https://docs.rs/syn/1.0.75/syn/fn.parse_file.html) sounds like a good fit for
this, however the tokens it produces does not record
proper [spans](https://docs.rs/proc-macro2/1.0.28/proc_macro2/struct.Span.html), so we cannot use it
to find the position of `mod`.

Lockjaw handles this by using another AST
parser ([tree_sitter](https://crates.io/crates/tree-sitter)) to parse the file.

## Finding which `mod` a token is in

Position info of a token is encoded in
the [span](https://docs.rs/proc-macro2/1.0.28/proc_macro2/struct.Span.html) object, but currently it
is opaque. Lockjaw forcefully extract the data from
span's [`Debug`](https://doc.rust-lang.org/std/fmt/trait.Debug.html) representation, which contains
the token's byte range. No need to say this is an awful thing to do.

Once the byte position is know, it can be used to find the deepest enclosing `mod`.

## Handling file `mod`

Rust currently handles file `mod` as includes internally. It inserts the content of the file
directly into the token stream, and ends up in a giant stream for the whole crate. The consequence
of this is the byte position the `proc_marco` token actually have is shifted around by inserted
files, and will not match its byte position inside the file the 3p AST parser sees.

Fortunately the Lockjaw `proc_macro` has the span of the `prologue!()` macro itself, and it knows
the macro must appear only once inside the file. Lockjaw is able to inspect the AST to find the file
position of the `prologue!()` macro, and calculate the offset between file position and token
position.

One of the effect is Lockjaw has to ban file `mod` that appears after `prologue!()`, as it will
invalidate the offset. Theoretically Lockjaw can recursively calculate the size of the file `mod`,
but limiting the position of file `mod` does not seem too bad.