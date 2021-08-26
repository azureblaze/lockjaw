# Multibinding

Multibinding is a special type of binding in Lockjaw that allows duplicated bindings. Instead of
enforcing one binding per type, multibindings gather the bindings into a collection, allowing
"everything implementing a type" to be injected. This is especially useful to build a plugin system
where an unspecified amount of implementations can be handled.

Multibindings comes in 2 flavors, a [`Vec<T>` binding](vec.md) that simply collects everything, and
[`HashMap<K,V>` binding](map.md) where key collisions are checked at compile time.