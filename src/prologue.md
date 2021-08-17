Prepares lockjaw to be used in the current file. Must be called in every file that uses any lockjaw
attribute macros, and before such usage.

`mod` declarations without body must not be used after the `prologue!()`.

# parameters

The macro accepts comma separated string literals as parameters. The first
([file_path](#file_path)) is always required.

## file_path

Path to the current file, related to the `Cargo.toml` of the crate. For example, `"src/main.rs"`,
`"src/foo/bar.rs"`, `"test/foo_test.rs"`.

A test will be generated to ensure this string matches the actual file location. However, a wrong
value is likely to result in cryptic compile time error where lockjaw fails to find types.

## mod_override

**Optional** [Rust path](https://doc.rust-lang.org/reference/paths.html) from the crate root to the
module the file represents. `""` for crate root. For example, `src/foo/bar.rs` would
use `"foo::bar"`.

Lockjaw will attempt to derive the current mod path from [`file_path`](#file_path), however this is
not always possible in doctests or other context.

# Implementation notes

Once [proc_macro_span](https://github.com/rust-lang/rust/issues/54725) is stabilized this macro can
be made more ergonomic and less error-prone by using
[Span.source_file()](https://doc.rust-lang.org/proc_macro/struct.Span.html#method.source_file).
Currently using it will require the `procmacro2_semver_exempt` cfg flag which is infectious.

Internally this macro works by scanning the file to find out `mod` and `use` declarations in each
scope, combining with the path from the file name allows lockjaw to guess the full path of every
symbol. Without it users will need to specify fully qualified path in all lockjaw type usages. 

