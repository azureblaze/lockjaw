#![allow(dead_code)]

use crate::manifest_parser::parse_manifest;
use lockjaw_common::manifest::DepManifests;
use serde::Deserialize;
use std::collections::HashMap;
use std::process::Command;

macro_rules! log {
    ($($tokens: tt)*) => {
        for line in format!($($tokens)*).lines() {
            println!("cargo::warning={}", line)
        }
    }
}

pub(crate) use log;

#[derive(Deserialize, Debug, Default, Clone)]
struct CargoMetadata {
    packages: Vec<CargoMetadataPackage>,
    resolve: CargoResolve,
}

#[derive(Deserialize, Debug, Clone, Default)]
struct CargoMetadataPackage {
    name: String,
    id: String,
    manifest_path: String,
    dependencies: Vec<CargoMetadataDependency>,
    targets: Vec<CargoTarget>,
}

#[derive(Deserialize, Debug, Clone)]
struct CargoMetadataDependency {
    name: String,
    kind: Option<String>,
}

#[derive(Deserialize, Debug, Clone, Default)]
struct CargoTarget {
    name: String,
    kind: Vec<String>,
    src_path: String,
}

#[derive(Deserialize, Debug, Clone, Default)]
struct CargoResolve {
    nodes: Vec<CargoNode>,
    root: Option<String>,
}
#[derive(Deserialize, Debug, Clone, Default)]
struct CargoNode {
    id: String,
    deps: Vec<CargoNodeDep>,
}

#[derive(Deserialize, Debug, Clone, Default)]
struct CargoNodeDep {
    name: String,
    pkg: String,
    dep_kinds: Vec<CargoDepKind>,
    features: Option<Vec<String>>,
}
#[derive(Deserialize, Debug, Clone, Default)]
struct CargoDepKind {
    kind: Option<String>,
    target: Option<String>,
}

pub fn build_manifest() {
    let cargo_output = Command::new("cargo")
        .arg("metadata")
        .arg("--manifest-path")
        .arg(std::env::var("CARGO_MANIFEST_PATH").expect("missing manifest dir"))
        .arg("--filter-platform")
        .arg(std::env::var("TARGET").expect("missing TARGET"))
        .arg("--frozen")
        .output()
        .unwrap();

    let cargo_metadata_json = String::from_utf8(cargo_output.stdout).unwrap();

    log!("{}", String::from_utf8(cargo_output.stderr).unwrap());

    let cargo_metadata: CargoMetadata = serde_json::from_str(&cargo_metadata_json).unwrap();

    let toml_map: HashMap<String, CargoMetadataPackage> = cargo_metadata
        .packages
        .iter()
        .map(|entry| (entry.id.clone(), entry.clone()))
        .collect();
    let dep_map: HashMap<String, CargoNode> = cargo_metadata
        .resolve
        .nodes
        .iter()
        .map(|entry| (entry.id.clone(), entry.clone()))
        .collect();

    let package_name = std::env::var("CARGO_PKG_NAME").unwrap();
    log!("package_name: {}", package_name);
    let package_id = cargo_metadata
        .packages
        .iter()
        .find(|package| package.name == package_name)
        .unwrap()
        .id
        .clone();
    log!("package_id: {}", package_id);

    let toml = toml_map.get(&package_id).unwrap();
    let mut target_packages: HashMap<String, LockjawPackage> = HashMap::new();
    for target in &toml.targets {
        if target.kind == vec!["custom-build".to_string()] {
            continue;
        }
        target_packages.insert(
            target.name.clone(),
            LockjawPackage {
                id: toml.id.clone(),
                name: toml.name.clone(),
                src_path: target.src_path.clone(),
                direct_crate_deps: toml
                    .dependencies
                    .iter()
                    .filter(|dep| {
                        if target.kind.contains(&"test".to_string()) {
                            dep.kind == Some("dev".to_string())
                        } else {
                            dep.kind == None
                        }
                    })
                    .map(|dep| dep.name.clone())
                    .collect(),
            },
        );
    }
    log!("target packages:{:#?}", target_packages);

    let prod_packages = gather_lockjaw_packages(&package_id, &toml_map, &dep_map, true, false);
    log!("prod packages:{:#?}", prod_packages);
    let test_packages = gather_lockjaw_packages(&package_id, &toml_map, &dep_map, true, true);
    log!("test packages:{:#?}", test_packages);

    let dep_manifest = DepManifests {
        prod_manifest: prod_packages
            .iter()
            .map(|package| parse_manifest(package))
            .collect(),
        test_manifest: test_packages
            .iter()
            .map(|package| parse_manifest(package))
            .collect(),
        root_manifests: target_packages
            .iter()
            .map(|entry| (entry.0.clone(), parse_manifest(entry.1)))
            .collect(),
    };

    let dep_manifest_path = format!("{}dep_manifest.json", std::env::var("OUT_DIR").unwrap());

    std::fs::write(
        &dep_manifest_path,
        serde_json::to_string_pretty(&dep_manifest).expect("cannot serialize manifest"),
    )
    .expect("cannot write manifest");

    log!(
        "dep manifest written to file:///{}",
        dep_manifest_path.replace("\\", "/")
    );
    println!(
        "cargo::rustc-env=LOCKJAW_DEP_MANIFEST={}",
        &dep_manifest_path
    )
}

#[derive(Debug, Clone)]
enum LockjawPackageKind {
    Prod,
    Test,
}

#[derive(Debug, Clone)]
pub(crate) struct LockjawPackage {
    pub id: String,
    pub name: String,
    pub src_path: String,
    pub direct_crate_deps: Vec<String>,
}
fn gather_lockjaw_packages(
    id: &String,
    toml_map: &HashMap<String, CargoMetadataPackage>,
    dep_map: &HashMap<String, CargoNode>,
    root: bool,
    for_test: bool,
) -> Vec<LockjawPackage> {
    let mut result = Vec::<LockjawPackage>::new();
    let node = dep_map.get(id).unwrap();

    if !node.deps.iter().any(|dep| dep.name == "lockjaw") {
        return result;
    }
    let toml = toml_map.get(id).unwrap();
    let mut direct_crate_deps: Vec<String> = Vec::new();
    for dep in &toml.dependencies {
        if for_test {
            if dep.kind == Some("dev".to_string()) {
                direct_crate_deps.push(dep.name.clone());
            }
        } else {
            if dep.kind == None {
                direct_crate_deps.push(dep.name.clone());
            }
        }
    }

    if !root {
        let target = toml
            .targets
            .iter()
            .find(|target| target.kind.contains(&"lib".to_string()))
            .expect(&format!("no lib target for {}", toml.name));

        result.push(LockjawPackage {
            id: node.id.clone(),
            name: toml.name.clone(),
            src_path: target.src_path.clone(),
            direct_crate_deps,
        });
    }

    for dep in &node.deps {
        if dep.name == "lockjaw" {
            continue;
        }

        if !dep.dep_kinds.iter().any(|kind| {
            kind.kind
                == if for_test {
                    Some("dev".to_string())
                } else {
                    None
                }
        }) {
            continue;
        }

        result.extend(gather_lockjaw_packages(
            &dep.pkg, toml_map, dep_map, false, for_test,
        ));
    }

    result
}
