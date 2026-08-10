use std::collections::{BTreeSet, VecDeque};
use std::path::{Path, PathBuf};
use std::process::Command;
use syn::{Item, Meta, Type};

pub(super) fn rust_sources(dir: &Path) -> Vec<PathBuf> {
    let repo = git_repo_root(dir);
    let output = Command::new("git")
        .args([
            "-C",
            repo.to_str().expect("repository root must be UTF-8"),
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "--",
            "crates/no-mistakes/src",
        ])
        .output()
        .expect("git ls-files must be available for docs coverage");
    assert!(
        output.status.success(),
        "git ls-files failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut paths = String::from_utf8(output.stdout)
        .expect("git ls-files output must be UTF-8")
        .lines()
        .filter_map(|relative| {
            let path = repo.join(relative);
            (path.extension().and_then(|ext| ext.to_str()) == Some("rs")
                && std::fs::symlink_metadata(&path)
                    .map(|metadata| metadata.file_type().is_file())
                    .unwrap_or(false))
            .then_some(path)
        })
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

fn git_repo_root(dir: &Path) -> PathBuf {
    let output = Command::new("git")
        .args([
            "-C",
            dir.to_str().expect("source directory must be UTF-8"),
            "rev-parse",
            "--show-toplevel",
        ])
        .output()
        .expect("git rev-parse must be available for docs coverage");
    assert!(
        output.status.success(),
        "git rev-parse failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    PathBuf::from(
        String::from_utf8(output.stdout)
            .expect("git repository root must be UTF-8")
            .trim(),
    )
}

pub(super) fn subcommand_enums(source: &str) -> Vec<(String, String)> {
    syn::parse_file(source)
        .expect("Rust sources must parse before extracting clap commands")
        .items
        .into_iter()
        .filter_map(|item| {
            let Item::Struct(item) = item else {
                return None;
            };
            let parent = item.ident.to_string();
            let fields = match item.fields {
                syn::Fields::Named(fields) => fields.named,
                _ => return None,
            };
            fields.into_iter().find_map(|field| {
                if !field.attrs.iter().any(is_subcommand_attribute) {
                    return None;
                }
                let Type::Path(path) = field.ty else {
                    panic!("clap subcommand field must have a named type");
                };
                Some((
                    parent.clone(),
                    path.path
                        .segments
                        .last()
                        .expect("clap subcommand type must have a path")
                        .ident
                        .to_string(),
                ))
            })
        })
        .collect()
}

fn is_subcommand_attribute(attribute: &syn::Attribute) -> bool {
    let Meta::List(list) = &attribute.meta else {
        return false;
    };
    attribute.path().is_ident("command")
        && list
            .tokens
            .to_string()
            .split(',')
            .any(|part| part.trim() == "subcommand")
}

pub(super) fn enum_block<'a>(source: &'a str, enum_name: &str) -> Option<&'a str> {
    let marker = format!("enum {enum_name}");
    let mut search_from = 0;
    while let Some(relative) = source[search_from..].find(&marker) {
        let start = search_from + relative;
        let after_name = start + marker.len();
        if source[after_name..]
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_alphanumeric() || character == '_')
        {
            search_from = after_name;
            continue;
        }
        let open = source[after_name..].find('{')? + after_name;
        let mut depth = 0;
        for (offset, character) in source[open..].char_indices() {
            match character {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(&source[open + 1..open + offset]);
                    }
                }
                _ => {}
            }
        }
        return None;
    }
    None
}

pub(super) fn enum_variants(block: &str) -> Vec<String> {
    block
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
                return None;
            }
            let name = line
                .split(|character: char| {
                    character == '('
                        || character == '{'
                        || character == ','
                        || character.is_whitespace()
                })
                .next()?;
            if name.chars().next()?.is_ascii_uppercase()
                && name
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '_')
            {
                Some(name.to_string())
            } else {
                None
            }
        })
        .collect()
}

pub(super) fn reachable_cli_pages(cli_dir: &Path) -> BTreeSet<String> {
    let cli_dir = cli_dir.canonicalize().unwrap();
    let mut seen = BTreeSet::new();
    let mut pending = VecDeque::from([cli_dir.join("README.md")]);
    while let Some(path) = pending.pop_front() {
        let Ok(relative) = path.strip_prefix(&cli_dir) else {
            continue;
        };
        let relative = relative.to_string_lossy().into_owned();
        if !seen.insert(relative) {
            continue;
        }
        let body = super::read(&path);
        let mut remaining = body.as_str();
        while let Some(start) = remaining.find("](") {
            remaining = &remaining[start + 2..];
            let Some(end) = remaining.find(')') else {
                break;
            };
            let target = remaining[..end].split('#').next().unwrap_or_default();
            remaining = &remaining[end + 1..];
            if target.is_empty() || target.starts_with("http") {
                continue;
            }
            let target_path = path.parent().unwrap().join(target);
            if target_path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                continue;
            }
            let Ok(target_path) = target_path.canonicalize() else {
                continue;
            };
            if target_path.starts_with(&cli_dir) {
                pending.push_back(target_path);
            }
        }
    }
    seen
}

pub(super) fn kebab_case(value: &str) -> String {
    let mut result = String::new();
    for (index, character) in value.chars().enumerate() {
        if character.is_uppercase() && index != 0 {
            result.push('-');
        }
        result.extend(character.to_lowercase());
    }
    result
}

#[test]
fn parses_multiline_subcommand_fixture() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/docs-coverage/multiline-subcommand/fixture.rs");
    let source = std::fs::read_to_string(&fixture).unwrap();
    assert_eq!(
        subcommand_enums(&source),
        vec![("FixtureArgs".to_string(), "FixtureCommand".to_string())]
    );
}
