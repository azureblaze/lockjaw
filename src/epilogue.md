Resolves the dependency graph and generate component code. Must be called in the crate root
(`main.rs` or `lib.rs` if it has tests) outside any `mod`/functions. Libraries do not need to call `epilogue!()`

a unit test will be generated to ensure it is called in the correct file.

# Parameters

The macro accepts additional parameters in the form of identifiers.

## `debug_output`

Writes the `epilogue!()` output to a file and `include!()` it, instead of inserting a hygienic token
stream. This allows easier debugging of code generation issues.