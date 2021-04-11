Resolves the dependency graph and generate component code. Must be called in in the crate root
(`lib.rs` or `main.rs`), after any other lockjaw macros, and outside any `mod`/functions

a unit test will be generated to ensure it is called in the correct file.

# Parameters
The macro accepts additional parameters in the from of identifiers. Regular users should rarely
need to use these.

## `debug_output`
Writes the `epilogue!()` output to a file and `include!()` it, instead of inserting a hygienic
token stream. This allows easier debugging of code generation issues.