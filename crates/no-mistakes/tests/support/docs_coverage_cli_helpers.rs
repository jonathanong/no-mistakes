use std::collections::{BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

pub(super) fn rust_sources(dir: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            paths.extend(rust_sources(&path));
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            paths.push(path);
        }
    }
    paths.sort();
    paths
}

pub(super) fn subcommand_enums(source: &str) -> Vec<(String, String)> {
    let mut result = Vec::new();
    let mut search_from = 0;
    while let Some(relative) = source[search_from..].find("#[command(subcommand)]") {
        let attribute_start = search_from + relative;
        let after_attribute = attribute_start + "#[command(subcommand)]".len();
        let field_end = source[after_attribute..]
            .find('}')
            .map(|offset| after_attribute + offset)
            .unwrap_or(source.len());
        let field = &source[after_attribute..field_end];
        let Some(command_field) = field.find("command:") else {
            search_from = after_attribute;
            continue;
        };
        let enum_name = field[command_field + "command:".len()..]
            .split([',', ';', '}'])
            .next()
            .unwrap()
            .trim()
            .to_string();
        let parent = source[..attribute_start]
            .rfind("struct ")
            .and_then(|offset| {
                source[offset + "struct ".len()..]
                    .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
                    .next()
            })
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| panic!("clap subcommand attribute has no containing struct"))
            .to_string();
        result.push((parent, enum_name));
        search_from = after_attribute;
    }
    result
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
