/*
Copyright 2020 Google LLC

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
use crate::error::CompileError;
use proc_macro2::TokenStream;

pub fn current_crate() -> String {
    std::env::var("CARGO_CRATE_NAME").expect("missing crate name env var")
}

pub fn cargo_manifest_dir() -> String {
    std::env::var("CARGO_MANIFEST_DIR")
        .expect("missing manifest dir env var")
        .replace("\\", "/")
}

/// Returns the common artifacts directory where all crates can read/write.
///
/// Each crate will write its [lockjaw_output_dir] here so dependent crate can find them.
pub fn proc_artifact_dir() -> String {
    return format!("{}/lockjaw_artifacts/", env!("PROC_ARTIFACT_DIR"));
}

/// Returns the output directory for the current crate.
pub fn lockjaw_output_dir() -> Result<String, TokenStream> {
    let out_dir = std::env::var("OUT_DIR")
        .map_compile_error("output dir not found. Call lockjaw::build_script in build.rs")?;
    return Ok(format!("{}/lockjaw/", out_dir));
}
