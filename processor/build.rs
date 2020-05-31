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

fn main() {
    if rustc_version::version_meta()
        .expect("cannot find rustc version")
        .channel
        == rustc_version::Channel::Nightly
    {
        println!("cargo:rustc-cfg=nightly");
    } else {
        println!("cargo:rustc-cfg=stable");
    }

    #[cfg(feature = "proto_generation")]
    protoc_rust::Codegen::new()
        .protoc_path(protoc_bin_vendored::protoc_bin_path().unwrap())
        .out_dir("src/protos")
        .inputs(&["protos/manifest.proto"])
        .include("protos")
        .run()
        .expect("protoc");

    println!(
        "cargo:rustc-env=PROC_ARTIFACT_DIR={}",
        std::env::var("OUT_DIR").expect("cannot find out dir")
    )
}
