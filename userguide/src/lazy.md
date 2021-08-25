# Lazy

[`Lazy<T>`](https://docs.rs/lockjaw/0.2.0/lockjaw/struct.Lazy.html) is a wrapper around
a [`Provider<T>`](https://docs.rs/lockjaw/0.2.0/lockjaw/struct.Provider.html), which creates the
object once and caches the result. The object will only be created
when [`get()`](https://docs.rs/lockjaw/0.2.0/lockjaw/struct.Lazy.html#method.get) is called, and
subsequent invocations returns a reference to the same object.