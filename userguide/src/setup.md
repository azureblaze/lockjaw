# Project Setup

Add lockjaw to your `Cargo.toml`:

```
lockjaw = "*"
```

:warning: **Lockjaw is still in early development.** all APIs are subjected to change and will break
without notice. You may want to pin it to a specific version, but if you don't want to fix breaking
changes, you probably should not use lockjaw yet.

The proc_macro and runtime library are packaged into the same crate, so this is the only target you
need.

Lockjaw also needs some environment setup, and requires a
[build script](https://doc.rust-lang.org/cargo/reference/build-scripts.html). Add `build.rs` next to
`Cargo.toml`, and call `lockjaw::build_script()` in `main()` inside it:

```rust
// build.rs
fn main(){
    lockjaw::build_script();
}
```

Lockjaw will ask you to do this if this step is missing.

You also must call the `lockjaw::epilogue!()` macro in the root of your crate (`lib.rs` or
`main.rs`) after all other uses of lockjaw, preferably at the end of the file.
