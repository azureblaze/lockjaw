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
fn injectable_non_struct() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/injectable/injectable_non_struct.rs",
        vec!["impl block expected"],
    )
}

#[test]
fn injectable_tuple() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/injectable/injectable_tuple.rs",
        vec!["impl block expected"],
    )
}

#[test]
fn injectable_no_inject() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/injectable/injectable_no_inject.rs",
        vec!["must have one method marked with #[inject]/#[factory]"],
    )
}

#[test]
fn injectable_multiple_inject() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/injectable/injectable_multiple_inject.rs",
        vec!["only one method can be marked with #[inject]/#[factory]"],
    )
}

#[test]
fn injectable_multiple_factory() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/injectable/injectable_multiple_factory.rs",
        vec!["only one method can be marked with #[inject]/#[factory]"],
    )
}

#[test]
fn injectable_inject_and_factory() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/injectable/injectable_inject_and_factory.rs",
        vec!["only one method can be marked with #[inject]/#[factory]"],
    )
}

#[test]
fn injectable_unknown_metadata() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/injectable/injectable_unknown_metadata.rs",
        vec!["unknown key: foo"],
    )
}

#[test]
fn injectable_factory_unknown_metadata() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/injectable/injectable_factory_unknown_metadata.rs",
        vec!["unknown key: foo"],
    )
}

#[test]
fn injectable_factory_implementing_not_path() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/injectable/injectable_factory_implementing_not_path.rs",
        vec!["path expected for 'implementing'"],
    )
}

#[test]
fn injectable_factory_implementing_not_trait() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/injectable/injectable_factory_implementing_not_trait.rs",
        vec!["expected trait, found struct `B`"],
    )
}

#[test]
fn injectable_factory_implementing_method_name_mismatch() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/injectable/injectable_factory_implementing_method_name_mismatch.rs",
        vec!["method `new` is not a member of trait `B`"],
    )
}

#[test]
fn injectable_factory_implementing_method_return_type_mismatch() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/injectable/injectable_factory_implementing_method_return_type_mismatch.rs",
        vec!["method `new` has an incompatible type for trait"],
    )
}

#[test]
fn injectable_container_not_path() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/injectable/injectable_container_not_path.rs",
        vec!["path expected for 'container'"],
    )
}

#[test]
fn injectable_container_not_scoped() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/injectable/injectable_container_not_scoped.rs",
        vec![
            "the 'container' metadata should only be used with an injectable that also has 'scope'",
        ],
    )
}
