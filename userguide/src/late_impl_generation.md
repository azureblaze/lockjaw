# Late implementation generation

Rust requires the `impl` block to appear in the same `mod` is the type it implements. Items cannot
be added to a mod later either. However with Lockjaw information to generate the implementation may
not be available at the time, especially for component builders. Furthermore, the implementation may
not even be possible in the same crate
for [`#[define_component]`](https://docs.rs/lockjaw/0.2.0/lockjaw/attr.define_component.html).

Component builders should be associated method of the component, or at least a free-standing
function in the same `mod`. Otherwise it will be hard for users to locate them.

Lockjaw handles this by implementing such methods by calling an `extern` method. Which will later
be generated. While this works, if the user forgets to call the code generation macro, a cryptic
linker error about missing symbol will appear.