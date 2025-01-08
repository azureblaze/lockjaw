#![allow(dead_code)]

pub fn build_manifest() {
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
