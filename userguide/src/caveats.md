# Caveats

Due to limitations of Rust and the author's ignorance, some of lockjaw's APIs have bad ergonomics,
and some others have abusive implementation which may break any time as Rust evolves
(or even already broken on some platforms). Beware of these before using lockjaw.

## Abusive

These are things lockjaw do that are probably not recommended, and may easily break in the future.

### Persistent mutable state between proc_macro invocations

Lockjaw proc_macros are not independent. Most macros only gather type data in memory, which is then
compiled to a dependency graph and used to generate the component codes.

Rust does not have any guarantee when proc_macros are executed, the order they are executed, the
threads they are executed on, or even the process they are executed on. This can easily break if the
compilation mode changes, such as incremental compilation (some proc_marco are not re-invoked) and
multi-threaded compilation (lockjaw macros has strict order dependency. For example `epilogue!()`
must be the last invoked lockjaw macro).

It would be nice if rust somehow allows proc_macro to generate pieces of serializable metadata that
can be read by other proc_macros.

### proc_macro writes data to build script OUT_DIR

To handle cross-crate dependencies, lockjaw writes the bindings metadata to OUT_DIR so other crates
can read it later.

Rust only specifies the build script should write to OUT_DIR, while proc_macro seems to run on the
same process and has the environment variable (when there is a build script, so lockjaw forces the
user to use a build script even though it is doing nothing).

### proc_macro reads data from disk

Same as above, lockjaw proc_macro reads data from other crates output so cross-crate dependency
graph can be generated.

### proc_macro writes data to other crate's OUT_DIR

Also for cross-crate dependency, the bindings metadata from other crates are required. While lockjaw
can determine the crate's dependencies with `cargo tree` (proc_macro spawning processes is probably
also a bad idea), it has no idea where their OUT_DIR are.

The current solution is to hardcode the proc_macro library's OUT_DIR into itself with a build
script, and let other crates that uses lockjaw also write their OUT_DIR to the proc_macro library's
OUT_DIR so it can be looked up later. Mutating data outside a crate's bound seems really bad.

This can be solved if:

* Rust allow looking up OUT_DIR of crates depended on.
* Rust allow compile time reflection such as inspecting whether a const string exists and reading
  its value, so the metadata can be encoded.

## Bad ergonomic

Some restrictions in the API causes additional work/confusion to users.

### Unable to resolve types

proc_macro are invoked at the token phase, so there are no type information available. Lockjaw is
unable to resolve `use` or type aliases, and must force users to provide fully qualified path
where ever it needs to compare types.

To fix this Rust need to provide some other code generation mechanism that runs at later phases
with type information.

### Unable to detect path to current item

Lockjaw need users to call `mod_epilogue!()` to provide the path of the current file, and add the
`path=""` metadata to some attributes, so it can figure out the fully qualified path of a binding.

This can be improved if [eager macro expansion](https://github.com/rust-lang/rfcs/pull/2320) is
implemented, where lockjaw can insert `module_path!()` where needed.