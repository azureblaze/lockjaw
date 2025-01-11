/*
Copyright 2025 Google LLC

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

#![allow(dead_code)]

pub(crate) fn build_manifest() {
    let dep_manifest = lockjaw_common::manifest_parser::build_manifest();

    let dep_manifest_path = format!("{}/dep_manifest.json", std::env::var("OUT_DIR").unwrap());

    std::fs::write(
        &dep_manifest_path,
        serde_json::to_string_pretty(&dep_manifest).expect("cannot serialize manifest"),
    )
    .expect("cannot write manifest");

    /*log!(
        "dep manifest written to file:///{}",
        dep_manifest_path.replace("\\", "/")
    );*/
    println!(
        "cargo::rustc-env=LOCKJAW_DEP_MANIFEST={}",
        &dep_manifest_path
    )
}
