Must be called at the end of a non-root (not `lib.rs` or `main.rs`) file that uses lockjaw
To let it know a file has concluded. The path from the crate root to the current `mod` the file
represents must be passed in as a string literal. i.e.
* `src/foo.rs` => `mod_epilogue!("foo");`
* `src/bar/mod.rs` => `mod_epilogue!("bar");`
* `src/bar/baz.rs` => `mod_epilogue!("bar::baz");`

Lockjaw requires this information to resolve the path of the bindings in the current file.