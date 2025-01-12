# Late implementation generation

Rust requires the `impl` block to appear in the same `mod` is the type it implements. Items cannot
be added to a mod later either. However with Lockjaw information to generate the implementation may
not be available at the time, especially for component builders. Furthermore, the implementation may
not even be possible in the same crate
for [`#[define_component]`](https://docs.rs/lockjaw/0.2.0/lockjaw/attr.define_component.html).

Component builders should be associated method of the component, or at least a freestanding
function in the same `mod`. Otherwise, it will be hard for users to locate them.

Lockjaw handles this by implementing such methods by calling an `extern` method. Which will later
be generated. While this works, if the user forgets to call the code generation macro, a cryptic
linker error about missing symbol will appear.

In addition, some late implementation methods needs a unique name since it might clash with other components. A unique
name cannot be generated because the only thing we know is the local component name in a `proc_marco`, and a component
with the same name might exist under a different `mod`/`crate`. Instead, the generated code declares a
`static mut *const ()` address
under the same `mod` as the component which it will [transmute](https://doc.rust-lang.org/std/mem/fn.transmute.html)
when it needs to call the implementation. The late generated implementation will set this address to the real
implementation in the component's builder(implementation knows the full path of the address.). This is a constant
assignment and hopefully the compiler can optimize it
away. 