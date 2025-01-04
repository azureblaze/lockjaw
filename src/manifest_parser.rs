use crate::build_script::{log, LockjawPackage};
use lockjaw_common::manifest::{Manifest, TypeRoot};
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use syn::{Item, ItemUse, UseTree};

pub(crate) fn parse_manifest(lockjaw_package: &LockjawPackage) -> Manifest {
    parse_file(
        &Path::new(&lockjaw_package.src_path),
        "(src)",
        &Vec::new(),
        lockjaw_package,
    )
}

fn parse_file(
    src_path: &Path,
    name: &str,
    parents: &Vec<String>,
    lockjaw_package: &LockjawPackage,
) -> Manifest {
    log!("parsing {}: {:?}", lockjaw_package.name, src_path);
    let mut src = String::new();
    File::open(src_path)
        .expect("source  doesn't exist")
        .read_to_string(&mut src)
        .expect("unable to read source");

    if let Ok(syn_file) = syn::parse_file(&src) {
        let debug_out_name = format!(
            "{}/{}_{}_{}.json",
            std::env::var("OUT_DIR").unwrap().replace('\\', "/"),
            lockjaw_package.name,
            parents.join("_"),
            if name == "(src)" { "" } else { name }
        );
        log!("debug ast: file:///{}", &debug_out_name);
        std::fs::write(&debug_out_name, format!("{:#?}", syn_file)).unwrap();

        parse_mods(src_path, name, &syn_file.items, parents, &lockjaw_package)
    } else {
        log!("{} is not valid rust", src_path.to_str().unwrap());
        Manifest::new()
    }
}

fn parse_mods(
    src_path: &Path,
    name: &str,
    items: &Vec<Item>,
    parents: &Vec<String>,
    lockjaw_package: &LockjawPackage,
) -> Manifest {
    let mut new_parents = parents.clone();
    if name.ne("(src)") {
        new_parents.push(name.to_owned());
    }

    let uses = get_uses(items, lockjaw_package, &new_parents);

    log!(
        "{}::{:?}::{} uses {:#?}",
        lockjaw_package.name,
        parents,
        name,
        uses
    );

    let mut result = Manifest::new();
    for item in items.iter() {
        match item {
            Item::Mod(item_mod) => {
                result.merge_from(&parse_mod_item(
                    src_path,
                    name,
                    item_mod,
                    &new_parents,
                    lockjaw_package,
                ));
            }
            _ => {}
        }
    }
    result
}

fn parse_mod_item(
    src_path: &Path,
    parent_name: &str,
    item_mod: &syn::ItemMod,
    parents: &Vec<String>,
    lockjaw_package: &LockjawPackage,
) -> Manifest {
    let mut result = Manifest::new();
    let mod_name = item_mod.ident.to_string();
    if let Some((_, items)) = &item_mod.content {
        result.merge_from(&parse_mods(
            src_path,
            &mod_name,
            items,
            &parents,
            lockjaw_package,
        ));
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
        ));
    }
    result
}

fn get_uses(
    items: &Vec<Item>,
    lockjaw_package: &LockjawPackage,
    parents: &Vec<String>,
) -> HashMap<String, UsePath> {
    let mut deps = HashSet::new();
    for dep in &lockjaw_package.direct_crate_deps {
        deps.insert(dep.clone());
    }
    // default deps
    deps.insert("std".to_owned());
    deps.insert("core".to_owned());
    let mut result = HashMap::<String, UsePath>::new();
    for item in items.iter() {
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
    result
}

struct UsePath {
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
    if use_item.leading_colon.is_some() || (segments.len() >= 1 && deps.contains(&segments[0])) {
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
