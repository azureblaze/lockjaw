Resolves the dependency graph and generate component code. Must be called in in the crate root
(`lib.rs` or `main.rs`), after any other lockjaw macros, and outside any `mod`/functions

a unit test will be generated to ensure it is called in the correct file.

# Parameters

The macro accepts additional parameters in the form of identifiers.

## `root`

Specify the current crate is a root crate for lockjaw. A root crate must exist when
[`#[define_component]`](define_component) or [`#[define_subcomponent]`](define_subcomponent), which
is where lockjaw will generate the actual component by gathering `install_in` [`#[module]`](module)
and
[`#[entry_point]`](entry_point) from the build dependency.

Typically, `root` is specified on a binary which won't ever be depended on by any other crates.
Compilation will fail if a crate using lockjaw depends directly or indirectly on a root crate, as
more bindings may be added to an existing component but it has already been generated.

## `debug_output`

Writes the `epilogue!()` output to a file and `include!()` it, instead of inserting a hygienic token
stream. This allows easier debugging of code generation issues.