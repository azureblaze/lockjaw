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
use compile_tests::set_src_path;

#[test]
fn graph() {
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/graph/graph_missing_binding.rs"),
            vec!["missing bindings for ::compile_tests_tests::Foo"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/graph/graph_cyclic_dependency.rs"),
            vec!["Cyclic dependency detected"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/graph/graph_cyclic_dependency_after_provider.rs"),
            vec![
                "Cyclic dependency detected",
                "Provider<::compile_tests_tests::Foo>",
                "::compile_tests_tests::S.foo",
            ],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/graph/graph_duplicated_binding.rs"),
            vec!["found duplicated bindings for ::compile_tests_tests::Foo"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/graph/graph_missing_module.rs"),
            vec![
                "module ::compile_tests_tests::Mm not found, required by ::compile_tests_tests::S",
            ],
        )
    }
}
