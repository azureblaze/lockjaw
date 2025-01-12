# Project Setup

## Cargo

Add lockjaw to the `[dependencies]` and `[build_dependencies]` section of your `Cargo.toml`:

```toml
{{ #include ../projects/setup/Cargo.toml:dep}}
```

The proc_macro and runtime library are packaged into the same crate, so this is the only target you
need. While the `proc_macro` library is heavy, Rust should be able to optimize them away in the
resulting binary. The runtime is pretty light, and the generated code is *supposed* to be zero cost
abstraction.

## Build script

Lockjaw also needs some environment setup, and requires a
[build script](https://doc.rust-lang.org/cargo/reference/build-scripts.html). Add `build.rs` next to
`Cargo.toml`, and
call [`lockjaw::build_script()`](https://docs.rs/lockjaw/latest/lockjaw/fn.build_script.html)
in `main()` inside it:

```rust,no_run,noplayground
// https://github.com/azureblaze/lockjaw/tree/main/userguide/projects/setup/build.rs
{{#include ../projects/setup/build.rs:main}}
```

The build script is required at the 'root' of your project, included binaries and any sub-crate with tests. Lockjaw will
ask you to do this if this step is missing.

The build script scans through all source under the crate and its dependencies to locate any bindings that should be a
part of the dependency graph. This is required as [path resolution](path_resolution.md) cannot be done in a `proc_macro`

## Epilogue macro

You also must call
the [`lockjaw::epilogue!()`](https://docs.rs/lockjaw/latest/lockjaw/macro.epilogue.html) macro in the
root of your root crate (`lib.rs` or
`main.rs`).

```rust,no_run,noplayground
// https://github.com/azureblaze/lockjaw/tree/main/userguide/projects/setup/src/main.rs
{{#include ../projects/setup/src/main.rs:epilogue}}
```

[Source](https://github.com/azureblaze/lockjaw/tree/main/userguide/projects/setup/) of this chapter