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
use crate::error::{spanned_compile_error, CompileError};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use regex::Regex;
use std::cell::RefCell;
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

    pub fn resolve_path(&self, identifier: &str, span: Span) -> Result<String, TokenStream> {
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
            return spanned_compile_error(span, &format!("{:?}, {:?}", span, self));
        }

        path.push_str(identifier);
        Ok(path)
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

pub fn resolve_path(identifier: &str, span: Span) -> Result<String, TokenStream> {
    SOURCE_DATA.with(|source_data| source_data.borrow().resolve_path(identifier, span))
}

pub fn handle_prologue(input: TokenStream) -> Result<TokenStream, TokenStream> {
    let args = literal_to_strings(&input)?;
    if args.len() == 0 {
        return spanned_compile_error(
            input.span(),
            "current source file name from cargo.toml expected",
        );
    }
    if args.len() > 2 {
        return spanned_compile_error(
            input.span(),
            "too many arguments. expected current source file name, and optional path to source root (src/, tests/, etc.)",
        );
    }

    let filename = args.get(0).unwrap().replace("\\", "/");
    let full_path = format!("{}/{}", cargo_manifest_dir(), filename);
    let mut source_file = File::open(&full_path).map_spanned_compile_error(
        input.span(),
        &format!("unable to find the source file at {}", &full_path),
    )?;

    let mod_path = if args.len() == 2 {
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
        .set_language(tree_sitter_rust::language())
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
        get_byte_offset(&ast, &src, input.span()),
    );

    let source_data = SourceData {
        base_path: mod_path.clone(),
        mods,
    };

    SOURCE_DATA.with(|data| {
        *data.borrow_mut() = source_data;
    });

    Ok(quote! {
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

fn get_byte_offset(ast: &Tree, src: &str, input_span: Span) -> usize {
    for n in ast.root_node().children(&mut ast.root_node().walk()) {
        let node: tree_sitter::Node = n;
        if node.kind() == "macro_invocation" {
            if node
                .child_by_field_name("macro")
                .unwrap()
                .utf8_text(&src.as_bytes())
                .unwrap()
                == "lockjaw::prologue"
            {
                for i in 0..node.child_count() {
                    let child = node.child(i).unwrap();
                    if child.kind() == "token_tree" {
                        return to_range(input_span).start - child.byte_range().start;
                    }
                }
            }
        }
    }
    // happens in doctest
    log!("WARNING: unable to find position of lockjaw::prologue!()");
    return 0;
}

fn parse_mods(
    src: &str,
    name: &str,
    span: Range<usize>,
    cursor: &mut TreeCursor,
    parents: &Vec<String>,
    byte_offset: usize,
) -> Vec<Mod> {
    let mut result = Vec::new();
    let mod_ = Mod {
        name: name.to_owned(),
        span: Range {
            start: span.start + byte_offset,
            end: span.end + byte_offset,
        },
        parents: parents.clone(),
    };
    result.push(mod_);
    let mut new_parents = parents.clone();
    if name.ne("(src)") {
        new_parents.push(name.to_owned());
    }
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
                        ));
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
