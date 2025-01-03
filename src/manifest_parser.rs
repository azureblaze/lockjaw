use crate::build_script::{log, LockjawPackage};
use lockjaw_common::manifest::{Manifest, TypeRoot};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tree_sitter::TreeCursor;

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
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .expect("cannot load rust parser");
    let mut src = String::new();
    File::open(src_path)
        .expect("source  doesn't exist")
        .read_to_string(&mut src)
        .expect("unable to read source");
    let ast = parser
        .parse(&src, None)
        .expect("source file is not valid rust");

    let mut result = Manifest::new();
    result.merge_from(&parse_mods(
        &src,
        src_path,
        name,
        &mut ast.walk(),
        parents,
        &lockjaw_package,
    ));

    result
}

fn parse_mods(
    src: &str,
    src_path: &Path,
    name: &str,
    cursor: &mut TreeCursor,
    parents: &Vec<String>,
    lockjaw_package: &LockjawPackage,
) -> Manifest {
    let mut new_parents = parents.clone();
    if name.ne("(src)") {
        new_parents.push(name.to_owned());
    }

    let uses = get_uses(&cursor.node(), src, lockjaw_package, &new_parents);

    log!(
        "{}::{:?}::{} uses {:#?}",
        lockjaw_package.name,
        parents,
        name,
        uses
    );

    let mut result = Manifest::new();
    if cursor.goto_first_child() {
        loop {
            match cursor.node().kind() {
                "mod_item" => {
                    result.merge_from(&parse_mod_item(
                        src,
                        src_path,
                        name,
                        cursor,
                        &new_parents,
                        lockjaw_package,
                    ));
                }
                _ => {}
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }
    result
}

fn parse_mod_item(
    src: &str,
    src_path: &Path,
    name: &str,
    cursor: &mut TreeCursor,
    parents: &Vec<String>,
    lockjaw_package: &LockjawPackage,
) -> Manifest {
    //log!("{}", cursor.node().to_sexp());
    let mut result = Manifest::new();
    let mod_name = cursor
        .node()
        .child_by_field_name("name")
        .unwrap()
        .utf8_text(src.as_bytes())
        .unwrap()
        .to_owned();
    if let Some(declaration_list) = cursor.node().child_by_field_name("body") {
        result.merge_from(&parse_mods(
            src,
            src_path,
            &mod_name,
            &mut declaration_list.walk(),
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
            dir.join(format!("{}/{}.rs", name, mod_name)),
            dir.join(format!("{}/mod.rs", mod_name)),
        ];
        let mod_path = candidates
            .iter()
            .find(|path| path.exists())
            .expect(&format!("cannot find any of {:?}", candidates));
        let mut mod_parents = parents.clone();
        if name.ne("(src)") {
            mod_parents.push(name.to_owned());
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
    node: &tree_sitter::Node,
    src: &str,
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
    for i in 0..node.child_count() {
        let child = node.child(i).unwrap();
        if child.kind() == "use_declaration" {
            result.extend(process_use(
                child
                    .child_by_field_name("argument")
                    .unwrap()
                    .utf8_text(src.as_bytes())
                    .unwrap(),
                &deps,
                parents,
                lockjaw_package,
            ))
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
    use_path: &str,
    deps: &HashSet<String>,
    parents: &Vec<String>,
    lockjaw_package: &LockjawPackage,
) -> HashMap<String, UsePath> {
    let mut result = HashMap::<String, UsePath>::new();
    let pattern = Regex::new(
        r"^(?P<global_prefix>::)?(?P<segments>(?:\w[\w\d_]*::)*)(?P<remainder>\{[\w\d\s,]*\}|[\w\d\s,]*)$",
    ).unwrap();
    let captures = pattern
        .captures(use_path)
        .expect(&format!("unable to handle use expression {}", use_path));
    let segments = captures
        .name("segments")
        .map(|m| {
            m.as_str()
                .split("::")
                .filter(|s| !s.is_empty())
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or(Vec::new());

    let mut path: Vec<String> = Vec::new();
    let type_root;
    if captures.name("global_prefix").is_some()
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
    let items = get_use_items(captures.name("remainder").unwrap().as_str());
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

struct UseItem {
    pub item: String,
    pub name: String,
}

fn get_use_items(remainder: &str) -> Vec<UseItem> {
    let item_strings = if !remainder.starts_with("{") {
        vec![remainder.to_owned()]
    } else {
        remainder
            .strip_prefix("{")
            .unwrap()
            .strip_suffix("}")
            .unwrap()
            .split(",")
            .filter(|s| !s.is_empty())
            .map(str::trim)
            .map(str::to_owned)
            .collect()
    };
    let mut result = Vec::new();
    for item_string in item_strings {
        if item_string == "*" {
            log!("WARNING: lockjaw is unable to handle * imports");
            continue;
        }
        if item_string.contains(" as ") {
            let split: Vec<_> = item_string.split(" as ").collect();
            result.push(UseItem {
                item: split[0].to_owned(),
                name: split[1].to_owned(),
            })
        } else {
            result.push(UseItem {
                item: item_string.clone(),
                name: item_string,
            })
        }
    }
    result
}
