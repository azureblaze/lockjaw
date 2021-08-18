Declares that a `Vec<T>` or `HashMap<K,V>` is a multibinding.

If [`#[into_vec]`](#into_vec)/[`#[elements_into_vec]`](#elements_into_vec)/
[`#[into_map]`](#into_map) exists in the same graph this is not necessary, but if the collection is
empty lockjaw needs to know that it is indeed a multibinding collection that is currently empty,
instead of the user trying to depend on a type that is not bound.