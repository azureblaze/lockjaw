use crate::attributes;
use crate::attributes::cfg::CfgEval;
use crate::log;
use crate::manifest::{ComponentType, DepManifests, Manifest, RootManifest, TypeRoot};
use crate::parsing::find_attribute;
use crate::type_data;
use crate::type_data::TypeData;
use anyhow::{bail, Context, Result};
use proc_macro2::TokenStream;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use syn::__private::ToTokens;
use syn::{Attribute, Item, ItemUse, Meta, UseTree};

#[derive(Deserialize, Debug, Default, Clone)]
struct CargoMetadata {
    packages: Vec<CargoMetadataPackage>,
    resolve: CargoResolve,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone, Default)]
struct CargoNodeDep {
    name: String,
    pkg: String,
    dep_kinds: Vec<CargoDepKind>,
    features: Option<Vec<String>>,
}
#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone, Default)]
struct CargoDepKind {
    kind: Option<String>,
    target: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum LockjawPackageKind {
    Prod,
    Test,
}

pub fn build_manifest() -> DepManifests {
    let cargo_output = Command::new("cargo")
        .arg("metadata")
        .arg("--manifest-path")
        .arg(std::env::var("CARGO_MANIFEST_PATH").expect("missing manifest dir"))
        //.arg("--filter-platform")
        //.arg(std::env::var("TARGET").expect("missing TARGET"))
        .arg("--format-version")
        .arg("1")
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
                direct_prod_crate_deps: toml
                    .dependencies
                    .iter()
                    .filter(|dep| dep.kind == None)
                    .map(|dep| dep.name.clone())
                    .collect(),
                direct_test_crate_deps: toml
                    .dependencies
                    .iter()
                    .filter(|dep| dep.kind == Some("dev".to_string()))
                    .map(|dep| dep.name.clone())
                    .collect(),
            },
        );
    }
    //log!("target packages:{:#?}", target_packages);

    let prod_packages = gather_lockjaw_packages(&package_id, &toml_map, &dep_map, true, false);
    //log!("prod packages:{:#?}", prod_packages);
    let test_packages = gather_lockjaw_packages(&package_id, &toml_map, &dep_map, true, true);
    //log!("test packages:{:#?}", test_packages);

    DepManifests {
        crate_name: package_name,
        prod_manifest: prod_packages
            .iter()
            .map(|package| parse_manifest(package, false))
            .collect(),
        test_manifest: test_packages
            .iter()
            .map(|package| parse_manifest(package, true))
            .collect(),
        root_manifests: target_packages
            .iter()
            .map(|entry| {
                (
                    entry.0.clone(),
                    RootManifest {
                        prod_manifest: parse_manifest(entry.1, false),
                        test_manifest: parse_manifest(entry.1, true),
                    },
                )
            })
            .collect(),
    }
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
    let mut direct_prod_crate_deps: Vec<String> = Vec::new();
    let mut direct_test_crate_deps: Vec<String> = Vec::new();
    for dep in &toml.dependencies {
        if dep.kind == Some("dev".to_string()) {
            direct_test_crate_deps.push(dep.name.clone());
        }
        if dep.kind == None {
            direct_prod_crate_deps.push(dep.name.clone());
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
            direct_prod_crate_deps,
            direct_test_crate_deps,
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

#[derive(Debug, Clone)]
pub struct LockjawPackage {
    pub id: String,
    pub name: String,
    pub src_path: String,
    pub direct_prod_crate_deps: Vec<String>,
    pub direct_test_crate_deps: Vec<String>,
}
pub fn parse_manifest(lockjaw_package: &LockjawPackage, cfg_test: bool) -> Manifest {
    let result = parse_file(
        &Path::new(&lockjaw_package.src_path),
        "(src)",
        &Vec::new(),
        lockjaw_package,
        cfg_test,
    );
    result.unwrap_or_else(|err| {
        log!("{}", err);
        Manifest::new()
    })
}

fn parse_file(
    src_path: &Path,
    name: &str,
    parents: &Vec<String>,
    lockjaw_package: &LockjawPackage,
    cfg_test: bool,
) -> Result<Manifest> {
    log!("parsing {}: {:?}", lockjaw_package.name, src_path);
    let mut src = String::new();
    File::open(src_path)
        .with_context(|| "source  doesn't exist")?
        .read_to_string(&mut src)
        .with_context(|| "unable to read source")?;

    if let Ok(syn_file) = syn::parse_file(&src) {
        #[cfg(disabled)]
        if let Ok(out_dir) = std::env::var("OUT_DIR") {
            let debug_out_name = format!(
                "{}/{}_{}_{}.json",
                out_dir.replace('\\', "/"),
                lockjaw_package.name,
                parents.join("_"),
                if name == "(src)" { "" } else { name }
            );
            log!("debug ast: file:///{}", &debug_out_name);
            std::fs::write(&debug_out_name, format!("{:#?}", syn_file)).unwrap();
        }
        parse_mods(
            src_path,
            name,
            &syn_file.items,
            parents,
            &lockjaw_package,
            cfg_test,
        )
    } else {
        bail!("{} is not valid rust", src_path.to_str().unwrap());
    }
}

fn parse_mods(
    src_path: &Path,
    name: &str,
    items: &Vec<Item>,
    parents: &Vec<String>,
    lockjaw_package: &LockjawPackage,
    cfg_test: bool,
) -> Result<Manifest> {
    let mut new_parents = parents.clone();
    if name.ne("(src)") {
        new_parents.push(name.to_owned());
    }

    let uses = get_uses(items, lockjaw_package, &new_parents, cfg_test)?;
    let mod_ = Mod {
        crate_name: lockjaw_package.name.clone(),
        name: name.to_owned(),
        parents: parents.clone(),
        uses,
    };
    let mut result = Manifest::new();
    for item in items.iter() {
        let attrs = item_attrs(item);
        if let Some(cfg) = find_attribute(&attrs, "cfg") {
            if let Meta::List(meta_list) = &cfg.meta {
                if !attributes::cfg::handle_cfg(meta_list)?.eval(cfg_test) {
                    continue;
                }
            }
        }

        if let Item::Mod(item_mod) = item {
            let mod_manifests = &parse_mod_item(
                src_path,
                name,
                item_mod,
                &new_parents,
                lockjaw_package,
                cfg_test,
            )?;
            result.merge_from(&mod_manifests);
        }

        for attribute in attrs.iter() {
            let type_data = type_data::from_path(attribute.path(), &mod_)?;
            match type_data.canonical_string_path().as_str() {
                "::lockjaw::injectable" => {
                    result.merge_from(&attributes::injectables::handle_injectable_attribute(
                        attribute.parse_args().unwrap_or(TokenStream::new()),
                        item.to_token_stream(),
                        &mod_,
                    )?);
                }
                "::lockjaw::component_visible" => {
                    result.merge_from(
                        &attributes::component_visibles::handle_component_visible_attribute(
                            attribute.parse_args().unwrap_or(TokenStream::new()),
                            item.to_token_stream(),
                            &mod_,
                        )?,
                    );
                }
                "::lockjaw::component" => {
                    result.merge_from(&attributes::components::handle_component_attribute(
                        attribute.parse_args().unwrap_or(TokenStream::new()),
                        item.to_token_stream(),
                        ComponentType::Component,
                        false,
                        &mod_,
                    )?);
                }
                "::lockjaw::subcomponent" => {
                    result.merge_from(&attributes::components::handle_component_attribute(
                        attribute.parse_args().unwrap_or(TokenStream::new()),
                        item.to_token_stream(),
                        ComponentType::Subcomponent,
                        false,
                        &mod_,
                    )?);
                }
                "::lockjaw::define_component" => {
                    result.merge_from(&attributes::components::handle_component_attribute(
                        attribute.parse_args().unwrap_or(TokenStream::new()),
                        item.to_token_stream(),
                        ComponentType::Component,
                        true,
                        &mod_,
                    )?);
                }
                "::lockjaw::define_subcomponent" => {
                    result.merge_from(&attributes::components::handle_component_attribute(
                        attribute.parse_args().unwrap_or(TokenStream::new()),
                        item.to_token_stream(),
                        ComponentType::Subcomponent,
                        true,
                        &mod_,
                    )?);
                }
                "::lockjaw::builder_modules" => {
                    result.merge_from(&attributes::components::handle_builder_modules_attribute(
                        attribute.parse_args().unwrap_or(TokenStream::new()),
                        item.to_token_stream(),
                        &mod_,
                    )?);
                }
                "::lockjaw::entry_point" => {
                    result.merge_from(&attributes::entrypoints::handle_entry_point_attribute(
                        attribute.parse_args().unwrap_or(TokenStream::new()),
                        item.to_token_stream(),
                        &mod_,
                    )?);
                }
                _ => {}
            }
        }
    }
    Ok(result)
}

fn item_attrs(item: &Item) -> Vec<Attribute> {
    match item {
        Item::Const(i) => i.attrs.clone(),
        Item::Enum(i) => i.attrs.clone(),
        Item::ExternCrate(i) => i.attrs.clone(),
        Item::Fn(i) => i.attrs.clone(),
        Item::ForeignMod(i) => i.attrs.clone(),
        Item::Impl(i) => i.attrs.clone(),
        Item::Macro(i) => i.attrs.clone(),
        Item::Mod(i) => i.attrs.clone(),
        Item::Static(i) => i.attrs.clone(),
        Item::Struct(i) => i.attrs.clone(),
        Item::Trait(i) => i.attrs.clone(),
        Item::TraitAlias(i) => i.attrs.clone(),
        Item::Type(i) => i.attrs.clone(),
        Item::Union(i) => i.attrs.clone(),
        Item::Use(i) => i.attrs.clone(),
        Item::Verbatim(_) => Vec::new(),
        _ => Vec::new(),
    }
}

fn parse_mod_item(
    src_path: &Path,
    parent_name: &str,
    item_mod: &syn::ItemMod,
    parents: &Vec<String>,
    lockjaw_package: &LockjawPackage,
    cfg_test: bool,
) -> Result<Manifest> {
    let mut result = Manifest::new();
    let mod_name = item_mod.ident.to_string();
    if let Some((_, items)) = &item_mod.content {
        result.merge_from(&parse_mods(
            src_path,
            &mod_name,
            items,
            &parents,
            lockjaw_package,
            cfg_test,
        )?);
    } else {
        let mut dir = Path::new(&lockjaw_package.src_path)
            .parent()
            .unwrap()
            .to_owned();
        for package in parents {
            dir = dir.join(package)
        }
        let candidates = vec![
            dir.join(format!("{}.rs", mod_name)),
            dir.join(format!("{}/{}.rs", parent_name, mod_name)),
            dir.join(format!("{}/mod.rs", mod_name)),
        ];
        let mod_path = candidates
            .iter()
            .find(|path| path.exists())
            .expect(&format!("cannot find any of {:?}", candidates));
        let mut mod_parents = parents.clone();
        if parent_name.ne("(src)") {
            mod_parents.push(parent_name.to_owned());
        }

        result.merge_from(&parse_file(
            mod_path,
            &mod_name,
            &mod_parents,
            lockjaw_package,
            cfg_test,
        )?);
    }
    Ok(result)
}

fn get_uses(
    items: &Vec<Item>,
    lockjaw_package: &LockjawPackage,
    parents: &Vec<String>,
    cfg_test: bool,
) -> Result<HashMap<String, UsePath>> {
    let mut deps = HashSet::new();

    for dep in if cfg_test {
        &lockjaw_package.direct_test_crate_deps
    } else {
        &lockjaw_package.direct_prod_crate_deps
    } {
        deps.insert(dep.clone());
    }
    // default deps
    deps.insert("std".to_owned());
    deps.insert("core".to_owned());
    let mut result = HashMap::<String, UsePath>::new();
    for item in items.iter() {
        let attrs = item_attrs(item);
        if let Some(cfg) = find_attribute(&attrs, "cfg") {
            if let Meta::List(meta_list) = &cfg.meta {
                if !attributes::cfg::handle_cfg(meta_list)?.eval(cfg_test) {
                    continue;
                }
            }
        }

        if let Item::ExternCrate(extern_crate) = item {
            deps.insert(extern_crate.ident.to_string());
        }
        if let Item::Use(item_use) = item {
            result.extend(process_use(&item_use, &deps, parents, lockjaw_package))
        }
    }
    for dep in &deps {
        if !result.contains_key(dep) {
            result.insert(
                dep.clone(),
                UsePath {
                    crate_: dep.clone(),
                    path: dep.clone(),
                    root: TypeRoot::GLOBAL,
                },
            );
        }
    }
    Ok(result)
}
#[derive(Debug)]
pub struct Mod {
    pub crate_name: String,
    pub name: String,
    pub parents: Vec<String>,
    pub uses: HashMap<String, UsePath>,
}

impl Mod {
    pub fn resolve_declare_path(&self, identifier: &str) -> Result<String> {
        let mut path = String::new();
        if !self.parents.is_empty() {
            path.push_str(&self.parents.join("::"));
            path.push_str("::");
        }
        if self.name != "(src)" {
            path.push_str(&self.name);
            path.push_str("::");
        }

        path.push_str(identifier);
        Ok(path)
    }

    pub fn resolve_path(&self, identifier: &str) -> Option<TypeData> {
        if let Some(use_path) = self.uses.get(identifier) {
            let mut result = TypeData::new();
            result.field_crate = use_path.crate_.clone();
            result.path = use_path.path.clone();
            result.root = use_path.root.clone();
            Some(result)
        } else {
            // assume the path is local.
            let mut result = TypeData::new();
            result.field_crate = self.crate_name.clone();
            result.root = TypeRoot::CRATE;
            result.path.push_str(&self.parents.join("::"));
            if self.name != "(src)" {
                if !self.parents.is_empty() {
                    result.path.push_str("::");
                }
                result.path.push_str(&self.name);
                result.path.push_str("::");
            }
            result.path.push_str(identifier);
            Some(result)
        }
    }
}

pub struct UsePath {
    pub crate_: String,
    pub path: String,
    pub root: TypeRoot,
}

impl Debug for UsePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();
        if self.root == TypeRoot::CRATE {
            result.push_str("crate");
        }
        result.push_str("::");
        result.push_str(&self.path);
        write!(f, "{}", result)
    }
}

fn process_use(
    use_item: &ItemUse,
    deps: &HashSet<String>,
    parents: &Vec<String>,
    lockjaw_package: &LockjawPackage,
) -> HashMap<String, UsePath> {
    let mut result = HashMap::<String, UsePath>::new();
    let mut segments = Vec::<String>::new();
    let mut tree = &use_item.tree;
    let remainder = loop {
        match tree {
            UseTree::Path(path) => {
                segments.push(path.ident.to_string());
                tree = &path.tree;
            }
            _ => break tree,
        }
    };
    let mut path: Vec<String> = Vec::new();
    let type_root;
    //log!("deps {:?}", deps);
    //log!("segments {:?}", segments);
    if segments.is_empty() {
        type_root = TypeRoot::GLOBAL;
    } else if use_item.leading_colon.is_some()
        || (segments.len() >= 1 && deps.contains(&segments[0]))
    {
        type_root = TypeRoot::GLOBAL;
        path.extend(segments.clone());
    } else {
        type_root = TypeRoot::CRATE;
        path.extend(parents.clone());
        for i in 0..segments.len() {
            if segments[i] == "self" {
                continue;
            }
            if segments[i] == "super" {
                path.pop();
                continue;
            }
            if segments[i] == "crate" {
                path.clear();
                continue;
            }
            path.extend_from_slice(&segments[i..segments.len()]);
            break;
        }
    }
    let items = get_use_items(remainder);
    for item in items {
        if item.name.is_empty() {
            continue;
        };
        let crate_ = if type_root == TypeRoot::CRATE {
            lockjaw_package.name.clone()
        } else if segments.len() >= 1 {
            segments[0].clone()
        } else {
            item.item.clone()
        };
        let mut item_path: String = path.join("::");
        if item.item != "self" {
            if !path.is_empty() {
                item_path.push_str("::");
            }
            item_path.push_str(&item.item);
        }
        let name = if item.name.contains(" as ") {
            item.name.split(" as ").collect::<Vec<&str>>()[1]
        } else {
            &item.name
        };
        result.insert(
            name.to_owned(),
            UsePath {
                crate_,
                path: item_path,
                root: type_root.clone(),
            },
        );
    }
    result
}

#[derive(Debug)]
struct UseItem {
    pub item: String,
    pub name: String,
}

fn get_use_items(remainder: &UseTree) -> Vec<UseItem> {
    let mut result = Vec::new();
    match remainder {
        UseTree::Path(_) => {
            panic!("unexpected path");
        }
        UseTree::Name(name) => result.push(UseItem {
            item: name.ident.to_string(),
            name: name.ident.to_string(),
        }),
        UseTree::Rename(rename) => result.push(UseItem {
            item: rename.ident.to_string(),
            name: rename.rename.to_string(),
        }),
        UseTree::Glob(_) => {
            log!("WARNING: lockjaw is unable to handle * imports");
        }
        UseTree::Group(group) => {
            for item in group.items.iter() {
                match item {
                    UseTree::Name(name) => result.push(UseItem {
                        item: name.ident.to_string(),
                        name: name.ident.to_string(),
                    }),
                    UseTree::Rename(rename) => result.push(UseItem {
                        item: rename.ident.to_string(),
                        name: rename.rename.to_string(),
                    }),
                    _ => panic!("invalid use group item"),
                }
            }
        }
    }
    result
}
