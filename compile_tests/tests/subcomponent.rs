/*
Copyright 2021 Google LLC

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a.rs copy of the License at

    https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

#[test]
fn subcomponent_multibinding_conflicts_with_parent_collection_binding() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/subcomponent/subcomponent_multibinding_conflicts_with_parent_collection_binding.rs",
        vec!["found duplicated bindings for ::std::vec::Vec<i32>"],
    )
}

#[test]
fn subcomponent_map_key_conflicts_with_parent_map_key() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/subcomponent/subcomponent_map_key_conflicts_with_parent_map_key.rs",
        vec!["found duplicated key I32(1)"],
    )
}

#[test]
fn subcomponent_parent_not_defined_component() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/subcomponent/subcomponent_parent_not_defined_component.rs",
        vec![
            "#[subcomponent]",
            "but the component is not annotated with #[define_component] or #[define_subcomponent]",
        ],
    )
}
