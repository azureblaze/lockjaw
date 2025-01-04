/*
Copyright 2021 Google LLC

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

use crate::environment::cargo_manifest_dir;
use crate::error::{compile_error, spanned_compile_error, CompileError};
use crate::parsing;
use lockjaw_common::environment::current_crate;
use lockjaw_common::manifest::TypeRoot;
use lockjaw_common::type_data::TypeData;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use regex::Regex;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::Read;
use std::ops::Range;
use std::path::Path;
use syn::parse::Parser;
use syn::spanned::Spanned;
use tree_sitter::{Tree, TreeCursor};

#[derive(Default, Debug)]
pub struct SourceData {
    base_path: String,
    mods: Vec<Mod>,
}

impl SourceData {
    fn new() -> Self {
        Default::default()
    }

    pub fn resolve_declare_path(
        &self,
        identifier: &str,
        span: Span,
    ) -> Result<String, TokenStream> {
        let mut path = String::new();
        if !self.base_path.is_empty() {
            path.push_str(&self.base_path);
            path.push_str("::");
        }
        if let Some(mod_) = self.get_mod(span) {
            let parents = &mod_.parents;
            if !parents.is_empty() {
                path.push_str(&parents.join("::"));
                path.push_str("::");
            }
            if mod_.name != "(src)" {
                path.push_str(&mod_.name);
                path.push_str("::");
            }
        } else {
            return spanned_compile_error(
                span,
                &format!("Unable to resolve path to current location. Is lockjaw::prologue!() called before using any other lockjaw attributes?"),
            );
        }

        path.push_str(identifier);
        Ok(path)
    }

    pub fn resolve_path(&self, identifier: &str, span: Span) -> Option<TypeData> {
        if let Some(mod_) = self.get_mod(span) {
            if let Some(use_path) = mod_.uses.get(identifier) {
                let mut result = TypeData::new();
                result.field_crate = use_path.crate_.clone();
                result.path = use_path.path.clone();
                result.root = use_path.root.clone();
                Some(result)
            } else {
                // assume the path is local.
                let mut result = TypeData::new();
                result.field_crate = current_crate();
                result.root = TypeRoot::CRATE;
                if !self.base_path.is_empty() {
                    result.path.push_str(&self.base_path);
                    result.path.push_str("::");
                }
                result.path.push_str(&mod_.parents.join("::"));
                if mod_.name != "(src)" {
                    if !mod_.parents.is_empty() {
                        result.path.push_str("::");
                    }
                    result.path.push_str(&mod_.name);
                    result.path.push_str("::");
                }
                result.path.push_str(identifier);
                Some(result)
            }
        } else {
            None
        }
    }

    fn get_mod(&self, span: Span) -> Option<&Mod> {
        let mut result: Option<&Mod> = None;
        let range = to_range(span);
        for m in &self.mods {
            if result.is_some() && m.span.start < result.unwrap().span.start {
                continue;
            }
            if m.span.contains(&range.start) {
                result = Some(m);
            }
        }
        result
    }
}

#[derive(Debug)]
struct Mod {
    name: String,
    span: Range<usize>,
    parents: Vec<String>,
    uses: HashMap<String, UsePath>,
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

fn to_range(span: proc_macro2::Span) -> Range<usize> {
    let pattern = Regex::new(r"#\d+ bytes\((?P<start>\d+)..(?P<end>\d+)").unwrap();
    let debug = format!("{:?}", span);
    let capture = pattern.captures(&debug).unwrap();
    Range {
        start: capture.name("start").unwrap().as_str().parse().unwrap(),
        end: capture.name("end").unwrap().as_str().parse().unwrap(),
    }
}

thread_local! {
    static SOURCE_DATA :RefCell<SourceData> = RefCell::new(SourceData::new());
}
pub fn prologue_check(span: Span) -> TokenStream {
    let mod_size = get_parent_mods(span).len();
    let mut supers = quote! {};
    for _ in 0..mod_size {
        supers = quote! {super::#supers};
    }
    quote! {
        impl #supers lockjaw_error_call_prologue_macro_before_any_other_use {}
    }
}

pub fn get_parent_mods(span: Span) -> Vec<String> {
    SOURCE_DATA.with(|source_data| {
        if let Some(mod_) = source_data.borrow().get_mod(span) {
            let mut result = Vec::new();
            result.extend(mod_.parents.clone());
            if mod_.name != "(src)" {
                result.push(mod_.name.clone());
            }
            return result;
        }
        Vec::new()
    })
}

/// Resolve the full path when an item is declared. e.g. struct Foo {} => crate::bar::Foo
pub fn resolve_declare_path(identifier: &str, span: Span) -> Result<String, TokenStream> {
    SOURCE_DATA.with(|source_data| source_data.borrow().resolve_declare_path(identifier, span))
}

/// Resolve the full path when an foreign item is referenced in a path. e.g. foo::Bar => ::qux::foo::Bar.
pub fn resolve_path(identifier: &str, span: Span) -> Option<TypeData> {
    SOURCE_DATA.with(|source_data| source_data.borrow().resolve_path(identifier, span))
}

thread_local! {
    static TEST_SOURCE :RefCell<String> = RefCell::new(String::new());
}

pub fn get_test_source() -> String {
    TEST_SOURCE.with(|test_source| test_source.borrow().clone())
}

pub fn handle_prologue(input: TokenStream, mut for_test: bool) -> Result<TokenStream, TokenStream> {
    let args = literal_to_strings(&input)?;
    if args.len() == 0 {
        return spanned_compile_error(
            input.span(),
            "current source file name from cargo.toml expected",
        );
    }
    if args.len() > 3 {
        return spanned_compile_error(
            input.span(),
            "too many arguments. expected current source file name, and optional path to source root (src/, tests/, etc.)",
        );
    }

    let filename = args.get(0).unwrap().replace("\\", "/");
    let full_path = format!("{}/{}", cargo_manifest_dir(), filename);
    if args.get(2).unwrap() == "test" {
        for_test = true;
    }
    if for_test {
        TEST_SOURCE.with(|test_source| *test_source.borrow_mut() = full_path.clone())
    }

    let mut source_file = File::open(&full_path).map_spanned_compile_error(
        input.span(),
        &format!("unable to find the source file at {}", &full_path),
    )?;

    let mod_path = if args.len() >= 2 {
        args.get(1).unwrap().replace("\\", "/")
    } else {
        get_mod_path(&filename)?
    };

    let mut src = String::new();
    source_file
        .read_to_string(&mut src)
        .map_spanned_compile_error(input.span(), "Unable to read file")?;

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .map_compile_error("cannot load rust parser")?;
    let ast = parser
        .parse(&src, None)
        .map_compile_error("source file is not valid rust")?;

    let mods = parse_mods(
        &src,
        "(src)",
        ast.root_node().byte_range(),
        &mut ast.walk(),
        &Vec::new(),
        validate_prologue(&ast, &filename, &src, &mod_path, input.span(), for_test)?,
        &mod_path,
        for_test,
    )?;

    let source_data = SourceData {
        base_path: mod_path.clone(),
        mods,
    };

    SOURCE_DATA.with(|data| {
        *data.borrow_mut() = source_data;
    });

    Ok(quote! {
        #[doc(hidden)]
        struct lockjaw_error_call_prologue_macro_before_any_other_use;
        #[test]
        fn prologue_path_matches_macro_input(){
            let mut iter = module_path!().split("::");
            iter.next();
            let path = iter.collect::<Vec<&str>>().join("::");
            assert_eq!(path, #mod_path,
            "source path supplied to prologue!() does not match actual path");
        }
    })
}

/// when "mod foo;" is used, the content of foo.rs is inserted into the parent file, which means
/// spans in foo.rs won't match its byte position in the file. This method attempts to correct this
/// by finding the position the prologue macro invocation (which is the call site and we have a
/// proc_macro span) for it.
fn validate_prologue(
    ast: &Tree,
    src_path: &str,
    src: &str,
    base_path: &str,
    input_span: Span,
    for_test: bool,
) -> Result<usize, TokenStream> {
    let uses = get_uses(&ast.root_node(), src, base_path, &Vec::new(), for_test)?;
    let prologue = local_path(&uses, "prologue");
    let lockjaw_macros: HashSet<String> = local_path(&uses, "injectable")
        .iter()
        .chain(local_path(&uses, "component").iter())
        .chain(local_path(&uses, "module").iter())
        .chain(local_path(&uses, "qualifier").iter())
        .map(String::clone)
        .collect();
    let epilogue = local_path(&uses, "epilogue");

    let path_pattern = Regex::new(r"#\[(?P<path>[\w\d:_]+).*]").unwrap();

    let mut byte_offset = None;
    let mut epilogue_found = false;
    for n in ast.root_node().children(&mut ast.root_node().walk()) {
        let node: tree_sitter::Node = n;

        if node.kind() == "expression_statement" {
            for c in node.children(&mut node.walk()) {
                if c.kind() == "macro_invocation" {
                    let macro_name = c
                        .child_by_field_name("macro")
                        .unwrap()
                        .utf8_text(&src.as_bytes())
                        .unwrap();
                    if prologue.contains(macro_name) {
                        for i in 0..c.child_count() {
                            let child = c.child(i).unwrap();
                            if child.kind() == "token_tree" {
                                byte_offset = Some(
                                    to_range(input_span.clone()).start - child.byte_range().start,
                                );
                            }
                        }
                    } else if epilogue.contains(macro_name) {
                        epilogue_found = true;
                    }
                }
            }
        } else if node.kind() == "attribute_item" {
            let attr = path_pattern
                .captures(node.utf8_text(&src.as_bytes()).unwrap())
                .unwrap()
                .name("path")
                .unwrap()
                .as_str();
            if lockjaw_macros.contains(attr) {
                if byte_offset.is_none() {
                    return compile_error(&format!(
                        "lockjaw macro cannot appear before lockjaw::prologue!() ({}:{}:{})",
                        src_path,
                        node.start_position().row + 1,
                        node.start_position().column,
                    ));
                }
                if epilogue_found {
                    return compile_error(&format!(
                        "lockjaw macro cannot appear after lockjaw::epilogue!() ({}:{}:{})",
                        src_path,
                        node.start_position().row + 1,
                        node.start_position().column,
                    ));
                }
            }
        } else if node.kind() == "mod_item" {
            if byte_offset.is_some()
                && !epilogue_found
                && node.child_by_field_name("body").is_none()
            {
                return compile_error(&format!(
                    "file modules cannot appear between lockjaw::prologue!() and lockjaw::epilogue!() ({}:{}:{})",
                    src_path,
                    node.start_position().row + 1,
                    node.start_position().column,
                ));
            }
        }
    }
    if let Some(result) = byte_offset {
        Ok(result)
    } else {
        if for_test {
            // happens in doctest
            Ok(0)
        } else {
            spanned_compile_error(
                input_span,
                " unable to find position of lockjaw::prologue!()",
            )
        }
    }
}

fn local_path<'a>(map: &'a HashMap<String, UsePath>, lockjaw_item: &'a str) -> HashSet<String> {
    let full_path = format!("::lockjaw::{}", lockjaw_item);
    let mut result = HashSet::<String>::new();
    let mut lockjaw_name = "lockjaw".to_owned();
    for pair in map.iter() {
        if pair.1.path.eq(&full_path) {
            result.insert(pair.0.to_owned());
        } else if pair.1.path.eq("::lockjaw") {
            lockjaw_name = pair.0.to_owned();
        }
    }
    result.insert(lockjaw_item.to_owned());
    result.insert(format!("{}::{}", lockjaw_name, lockjaw_item));
    result
}

fn parse_mods(
    src: &str,
    name: &str,
    span: Range<usize>,
    cursor: &mut TreeCursor,
    parents: &Vec<String>,
    byte_offset: usize,
    base_path: &str,
    for_test: bool,
) -> Result<Vec<Mod>, TokenStream> {
    let mut new_parents = parents.clone();
    if name.ne("(src)") {
        new_parents.push(name.to_owned());
    }
    let mut result = Vec::new();
    let mod_ = Mod {
        name: name.to_owned(),
        span: Range {
            start: span.start + byte_offset,
            end: span.end + byte_offset,
        },
        parents: parents.clone(),
        uses: get_uses(&cursor.node(), src, base_path, &new_parents, for_test)?,
    };
    result.push(mod_);

    if cursor.goto_first_child() {
        loop {
            match cursor.node().kind() {
                "mod_item" => {
                    let mod_name = cursor
                        .node()
                        .child_by_field_name("name")
                        .unwrap()
                        .utf8_text(src.as_bytes())
                        .unwrap()
                        .to_owned();
                    if let Some(declaration_list) = cursor.node().child_by_field_name("body") {
                        result.extend(parse_mods(
                            src,
                            &mod_name,
                            cursor.node().byte_range(),
                            &mut declaration_list.walk(),
                            &new_parents,
                            byte_offset,
                            base_path,
                            for_test,
                        )?);
                    }
                }
                _ => {}
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }
    Ok(result)
}

fn get_uses(
    node: &tree_sitter::Node,
    src: &str,
    base_path: &str,
    parents: &Vec<String>,
    for_test: bool,
) -> Result<HashMap<String, UsePath>, TokenStream> {
    let mut deps = parsing::get_crate_deps(for_test, true);
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
                base_path,
                parents,
            )?)
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

fn process_use(
    use_path: &str,
    deps: &HashSet<String>,
    base_path: &str,
    parents: &Vec<String>,
) -> Result<HashMap<String, UsePath>, TokenStream> {
    let mut result = HashMap::<String, UsePath>::new();
    let pattern = Regex::new(
        r"^(?P<global_prefix>::)?(?P<segments>(?:\w[\w\d_]*::)*)(?P<remainder>\{[\w\d\s,]*\}|[\w\d\s,]*)$",
    ).unwrap();
    let captures = pattern
        .captures(use_path)
        .map_compile_error(&format!("unable to handle use expression {}", use_path))?;
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
        path.extend(
            base_path
                .split("::")
                .filter(|s| !s.is_empty())
                .map(str::to_owned)
                .collect::<Vec<_>>(),
        );
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
        let crate_ = if type_root == TypeRoot::CRATE {
            current_crate()
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
    Ok(result)
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

fn get_mod_path(file_path: &str) -> Result<String, TokenStream> {
    let source_root = get_source_root(&file_path).map_compile_error(
        "unable to infer source root. \
            pass in the source root as second arg if the project layout does not follow \
            https://doc.rust-lang.org/cargo/guide/project-layout.html",
    )?;

    if source_root != "src/" {
        return Ok("".to_owned());
    }

    let result = Path::new(file_path)
        .strip_prefix(Path::new(&source_root))
        .unwrap()
        .parent()
        .unwrap()
        .to_str()
        .unwrap()
        .replace("/", "::");
    let stem = Path::new(file_path).file_stem().unwrap().to_str().unwrap();
    match stem {
        "lib" | "main" | "mod" => Ok(result),
        _ => {
            if result.is_empty() {
                Ok(stem.to_owned())
            } else {
                Ok(format!("{}::{}", result, stem))
            }
        }
    }
}

#[test]
pub fn get_mod_path_main_empty() {
    assert_eq!(get_mod_path("src/main.rs").unwrap(), "");
}

#[test]
pub fn get_mod_path_lib_empty() {
    assert_eq!(get_mod_path("src/lib.rs").unwrap(), "");
}

#[test]
pub fn get_mod_path_file_file() {
    assert_eq!(get_mod_path("src/foo.rs").unwrap(), "foo");
}

#[test]
pub fn get_mod_path_mod_mod() {
    assert_eq!(get_mod_path("src/foo/mod.rs").unwrap(), "foo");
}

#[test]
pub fn get_mod_path_mod_file_file() {
    assert_eq!(get_mod_path("src/foo/bar.rs").unwrap(), "foo::bar");
}

fn get_source_root(file_path: &str) -> Option<String> {
    if file_path.starts_with("src/bin/") {
        return Some("src/bin/".to_owned());
    }
    if file_path.starts_with("src/") {
        return Some("src/".to_owned());
    }
    if file_path.starts_with("benches/") {
        return Some("benches/".to_owned());
    }
    if file_path.starts_with("examples/") {
        return Some("examples/".to_owned());
    }
    if file_path.starts_with("tests/") {
        return Some("tests/".to_owned());
    }
    None
}

fn literal_to_strings(input: &TokenStream) -> Result<Vec<String>, TokenStream> {
    let parser = syn::punctuated::Punctuated::<syn::LitStr, syn::Token![,]>::parse_terminated;
    let punctuated = parser
        .parse2(input.clone())
        .map_spanned_compile_error(input.span(), "comma seperated string literals expected")?;
    Ok(punctuated.iter().map(syn::LitStr::value).collect())
}
