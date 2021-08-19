# Project Setup

Add lockjaw to the `[dependencies]` and `[build_dependencies]` section of your `Cargo.toml`:

```toml
{{#include ../projects/setup/Cargo.toml:dep}}
```

The proc_macro and runtime library are packaged into the same crate, so this is the only target you
need.

Lockjaw also needs some environment setup, and requires a
[build script](https://doc.rust-lang.org/cargo/reference/build-scripts.html). Add `build.rs` next to
`Cargo.toml`, and
call [`lockjaw::build_script()`](https://docs.rs/lockjaw/0.2.0/lockjaw/fn.build_script.html)
in `main()` inside it:

```rust,no_run,noplayground
// https://github.com/azureblaze/lockjaw/tree/main/userguide/projects/setup/build.rs
{{#include ../projects/setup/build.rs:main}}
```

Lockjaw will ask you to do this if this step is missing.

You also must call
the [`lockjaw::epilogue!()`](https://docs.rs/lockjaw/0.2.0/lockjaw/macro.epilogue.html) macro in the
root of your crate (`lib.rs` or
`main.rs`) after all other uses of lockjaw, preferably at the end of the file.

```rust,no_run,noplayground
// https://github.com/azureblaze/lockjaw/tree/main/userguide/projects/setup/src/main.rs
{{#include ../projects/setup/src/main.rs:epilogue}}
```

[Source](https://github.com/azureblaze/lockjaw/tree/main/userguide/projects/setup/) of this chapter