#![allow(dead_code)]

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
    log!("toml:{:#?}", toml_map.get(&package_id).unwrap());
    log!("node:{:#?}", dep_map.get(&package_id).unwrap());
    let prod_packages = gather_lockjaw_packages(&package_id, &toml_map, &dep_map, false);
    log!("prod packages:{:#?}", prod_packages);
    let test_packages = gather_lockjaw_packages(&package_id, &toml_map, &dep_map, true);
    log!("test packages:{:#?}", test_packages);
}

#[derive(Debug)]
enum LockjawPackageKind {
    Prod,
    Test,
}

#[derive(Debug)]
struct LockjawPackage {
    id: String,
    name: String,
    src_path: String,
}
fn gather_lockjaw_packages(
    id: &String,
    toml_map: &HashMap<String, CargoMetadataPackage>,
    dep_map: &HashMap<String, CargoNode>,
    for_test: bool,
) -> Vec<LockjawPackage> {
    let mut result = Vec::<LockjawPackage>::new();
    let node = dep_map.get(id).unwrap();

    if !node.deps.iter().any(|dep| dep.name == "lockjaw") {
        return result;
    }
    let toml = toml_map.get(id).unwrap();
    let target = toml
        .targets
        .iter()
        .find(|target| target.kind.contains(&"lib".to_string()))
        .or_else(|| {
            toml.targets
                .iter()
                .find(|target| target.kind.contains(&"bin".to_string()))
        })
        .expect(&format!("no bin or lib target for {}", toml.name));

    result.push(LockjawPackage {
        id: node.id.clone(),
        name: toml.name.clone(),
        src_path: target.src_path.clone(),
    });

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
            &dep.pkg, toml_map, dep_map, for_test,
        ));
    }

    result
}
