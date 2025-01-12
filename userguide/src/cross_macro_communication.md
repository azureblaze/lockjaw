# Cross macro communication(obsolete)

<div class="warning">
This only applies to lockjaw 0.3. It has since been resolved in 0.3 by moving dependency gathering
to build scripts.
</div>

Lockjaw `proc_macro` are not independent, it requires inputs from other macros in strict order:

1. A first pass on the file to [generate mod info for path resolution](path_resolution.md)
2. Generating binding definition, with paths resolved to fully qualified path using info from step
    1.
3. Actual generation of the components, after all bindings are defined.

Additionally the binding info in step 2. may be needed across crates.

## Intra-crate macro communication

Lockjaw macros talk to each other by storing states in a static mutable variable. This requires the
token/macro phase of rust to invoke all macros in a single process, single thread, sequentially as
they appear in the source, as well as expanding file `mod` in place as soon as they are encountered.

Rust does not have any guarantee when proc_macros are executed, the order they are executed, the
threads they are executed on, or even the process they are executed on. This can easily break if the
compilation mode changes, such as incremental compilation (some proc_marco are not re-invoked) and
multi-threaded compilation.

It would be nice if rust somehow allows proc_macro to generate pieces of serializable metadata that
can be read by other proc_macros.

## Intra-crate macro communication

Lockjaw allows `#[inject]` in a crate to be used by another crate that depends on it. Additionally
`#[denfine_component]` allows components to be generated at the root crate, using binding info from
every crate it directly or indirectly depends on. This means a crate must be able to read the
binding metadata of every crate.

Lockjaw handles this by writing the binding metadata of each crate to a file, which other crates can
read. This creates a few issues:

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

Also for cross-crate dependency, the binding metadata from other crates are required. While lockjaw
can determine the crate's dependencies with `cargo tree` (proc_macro spawning processes is probably
also a bad idea), it has no idea where their OUT_DIR are.

The current solution is to hardcode the proc_macro library's OUT_DIR into itself with a build
script, and let other crates that uses lockjaw also write their OUT_DIR to the proc_macro library's
OUT_DIR so it can be looked up later. Mutating data outside a crate's bound seems really bad.

This can be solved if:

* Rust allow looking up OUT_DIR of crates depended on.
* Rust allow compile time reflection such as inspecting whether a const string exists and reading
  its value, so the metadata can be encoded.