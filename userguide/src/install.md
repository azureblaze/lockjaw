# Installing Modules

`#[injectables]` are global bindings, which mean they are included into any dependency graph. But
for `#[provides]` which bindings should be available should be specified, as it may be situational.

Each component should declare a list of modules they want to use with a  
using the `modules` field in the `#[component]` attribute.

```rust
#[component(modules=[MyModule])]
trait MyComponent {
    fn my_module_binding() -> MyModuleBinding;
}
```