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
fn file_mod_after_prologue_fail() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with("tests/prologue/file_mod_after_prologue.rs", vec!["error: file modules cannot appear between lockjaw::prologue!() and lockjaw::epilogue!()"])
}

#[test]
fn file_mod_after_epilogue_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/prologue/file_mod_after_epilogue.rs")
}

#[test]
fn file_mod_before_prologue_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/prologue/file_mod_before_prologue.rs")
}

#[test]
fn attr_before_prologue_fail() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/prologue/attr_before_prologue.rs",
        vec!["lockjaw macro cannot appear before lockjaw::prologue!()"],
    )
}

#[test]
fn attr_after_epilogue() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/prologue/attr_after_epilogue.rs",
        vec!["lockjaw macro cannot appear after lockjaw::epilogue!()"],
    )
}
