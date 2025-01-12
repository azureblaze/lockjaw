# Changes

## 0.3

* Dependency gathering is now done through the [build script](setup.md#build-script) instead of proc_macro.
    * This removes the need for [cross macro communication](cross_macro_communication.md) as all info is now readily
      available when proc_macro runs.
    * Sources are parsed twice once in the build script and once by the compiler. However, this is already required to
      perform [path resolution](path_resolution.md)
    * The proc_macro now only generate codes, using the result from the build script. The proc macro no longer need to
      resolve global path names.
    * Path resolution/dependency gathering now use [syn](https://crates.io/crates/syn) to parse the source instead of
      using [tree-sitter-rust](https://crates.io/crates/tree-sitter-rust) to be more consistent with how proc_macro
      parse the code.
* The `lockjaw::prologue!(path/to/src)` is removed.
    * The build script is able to directly infer the source path.
* No longer need to specify `root`/`test` in `lockjaw::epilogue`