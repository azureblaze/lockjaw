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
fn injectable() {
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/injectable/injectable_non_struct.rs"),
            vec!["impl block expected"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/injectable/injectable_tuple.rs"),
            vec!["impl block expected"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/injectable/injectable_no_inject.rs"),
            vec!["must have one method marked with #[inject]/#[factory]"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/injectable/injectable_multiple_inject.rs"),
            vec!["only one method can be marked with #[inject]/#[factory]"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/injectable/injectable_multiple_factory.rs"),
            vec!["only one method can be marked with #[inject]/#[factory]"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/injectable/injectable_inject_and_factory.rs"),
            vec!["only one method can be marked with #[inject]/#[factory]"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/injectable/injectable_unknown_metadata.rs"),
            vec!["unknown key: foo"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/injectable/injectable_factory_unknown_metadata.rs"),
            vec!["unknown key: foo"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/injectable/injectable_factory_implementing_not_path.rs"),
            vec!["path expected for 'implementing'"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/injectable/injectable_factory_implementing_not_trait.rs"),
            vec!["expected trait, found struct `B`"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path(
                "tests/injectable/injectable_factory_implementing_method_name_mismatch.rs",
            ),
            vec!["method `new` is not a member of trait `B`"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path(
                "tests/injectable/injectable_factory_implementing_method_return_type_mismatch.rs",
            ),
            vec!["method `new` has an incompatible type for trait"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/injectable/injectable_container_not_path.rs"),
            vec!["path expected for 'container'"],
        )
    }
    {
        let t = trybuild::TestCases::new();
        t.compile_failed_with(
            set_src_path("tests/injectable/injectable_container_not_scoped.rs"),
            vec![
                "the 'container' metadata should only be used with an injectable that also has 'scope'",
            ],
        )
    }
}
