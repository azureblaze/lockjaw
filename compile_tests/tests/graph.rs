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
fn graph_missing_binding() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/graph/graph_missing_binding.rs",
        vec!["missing bindings for ::$CRATE::Foo"],
    )
}

#[test]
fn graph_cyclic_dependency() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/graph/graph_cyclic_dependency.rs",
        vec!["Cyclic dependency detected"],
    )
}

#[test]
fn graph_cyclic_dependency_after_provider() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/graph/graph_cyclic_dependency_after_provider.rs",
        vec![
            "Cyclic dependency detected",
            "Provider<::$CRATE::Foo>",
            "::$CRATE::S.foo",
        ],
    )
}

#[test]
fn graph_duplicated_binding() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/graph/graph_duplicated_binding.rs",
        vec!["found duplicated bindings for ::$CRATE::Foo"],
    )
}

#[test]
fn graph_missing_module() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/graph/graph_missing_module.rs",
        vec!["module ::$CRATE::Mm not found, required by ::$CRATE::S"],
    )
}
