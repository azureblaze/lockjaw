# Before Using Lockjaw

Before using lockjaw you should carefully consider if it is really for you. Having compile time
dependency injection is wonderful, but the current Rust proc-macro system does not make it easy, and
there are a lot of trade-offs.

## Robustness

Lockjaw is experimental in its nature, with the main goal of reaching feature parity with Dagger. It
tries to answer:

* What dependency injection would look like in Rust?
    * Is it even possible?
    * Is it going to be useful?
    * How should lifetimes and borrows be handled?
* What are the hurdles when trying to implement a dependency injection framework in Rust?

Currently lockjaw focuses on "can" instead of "should". If there is any hacky undocumented compiler
behavior we can abuse to implement a feature, It **will** be used. While we try to make it bug-free
and generate safe code in the current version of Rust, there are no guarantee it will continue to
work in the future.

See the [caveats](caveats.md) section for all horrible hacks used.

Future efforts might be made to get language features Lockjaw need implemented in Rust, but that is
a very long road.

## Maintenance

Lockjaw is a quite complicated project, but it is built by the developer with short attention span
and other priorities, as a hobby project, while also trying to learn Rust. Do not expect continuous
support, and especially consider that newer Rust can break Lockjaw at any moment.

## Irreversibility

Dependency injection frameworks are very invasive and will change how code are written. If you
decide to remove Lockjaw in the future, you may need to rewrite the whole project.

## Only use lockjaw if...

* You also like to live dangerously.
* You are experimenting things.
* You are working on a small project, and it won't be maintained in the future.
* Someone else will be maintaining the project in the future, and you are a horrible individual.
* You love dependency injection so much you are willing to patch bugs yourself, and freeze Rust
  version if it breaks Lockjaw.