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
        vec!["must have one method marked with #[inject]"],
    )
}

#[test]
fn injectable_multiple_inject() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/injectable/injectable_multiple_inject.rs",
        vec!["only one method can be marked with #[inject]"],
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
