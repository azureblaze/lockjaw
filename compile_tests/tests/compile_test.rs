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
    t.compile_fail("tests/compile-fail/file_mod_after_prologue.rs")
}

#[test]
fn file_mod_after_epilogue_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/run-pass/file_mod_after_epilogue.rs")
}

#[test]
fn file_mod_before_prologue_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/run-pass/file_mod_before_prologue.rs")
}

#[test]
fn attr_before_prologue_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/attr_before_prologue.rs")
}

#[test]
fn attr_after_epilogue() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/attr_after_epilogue.rs")
}
