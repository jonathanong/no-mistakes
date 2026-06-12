use super::comments::{strip_comments, strip_sql_comments};
use super::literals::quoted_strings;
use super::object::{const_object_body, top_level_object_keys, top_level_property_values};
use super::ts_union;
use super::SetSpec;
use crate::codebase::ts_source::relative_slash_path;
use anyhow::Result;
use regex::Regex;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub(super) struct ExtractedSet {
    pub(super) file: String,
    pub(super) values: BTreeSet<String>,
}

pub(super) fn extract_set(
    root: &Path,
    spec: &SetSpec,
    files: &[PathBuf],
    target_roots: &[PathBuf],
) -> Result<ExtractedSet> {
    if spec.kind == "path-regex-capture" {
        return extract_path_regex_set(root, spec, files, target_roots);
    }
    let path = resolve_spec_file(root, &spec.file, target_roots);
    let source = std::fs::read_to_string(&path)?;
    let values = match spec.kind.as_str() {
        "ts-string-union" => extract_ts_string_union(&source, &spec.target),
        "ts-const-object-keys" => extract_ts_const_object_keys(&source, &spec.target),
        "ts-const-object-property" => {
            extract_ts_const_object_property(&source, &spec.target, &spec.property)
        }
        "sql-enum" => extract_sql_enum(&source, &spec.target),
        _ => BTreeSet::new(),
    };
    Ok(ExtractedSet {
        file: relative_slash_path(root, &path),
        values,
    })
}

fn resolve_spec_file(root: &Path, file: &str, target_roots: &[PathBuf]) -> PathBuf {
    let repo_path = root.join(file);
    if repo_path.exists() {
        return repo_path;
    }
    target_roots
        .iter()
        .filter(|target_root| *target_root != root)
        .map(|target_root| target_root.join(file))
        .find(|path| path.exists())
        .unwrap_or(repo_path)
}

pub(super) fn extract_path_regex_set(
    root: &Path,
    spec: &SetSpec,
    files: &[PathBuf],
    target_roots: &[PathBuf],
) -> Result<ExtractedSet> {
    let regex = Regex::new(&spec.pattern)?;
    let mut values = BTreeSet::new();
    for file in files {
        for rel in relative_paths_for_matching(root, file, target_roots) {
            let Some(captures) = regex.captures(&rel) else {
                continue;
            };
            let value = captures
                .name("value")
                .or_else(|| captures.get(1))
                .map(|capture| capture.as_str().to_string());
            values.extend(value);
        }
    }
    Ok(ExtractedSet {
        file: spec.file.clone().if_empty(".".to_string()),
        values,
    })
}

fn relative_paths_for_matching(root: &Path, file: &Path, target_roots: &[PathBuf]) -> Vec<String> {
    let mut paths = target_roots
        .iter()
        .filter(|target_root| file.starts_with(target_root))
        .map(|target_root| relative_slash_path(target_root, file))
        .collect::<Vec<_>>();
    let repo_rel = relative_slash_path(root, file);
    if !paths.contains(&repo_rel) {
        paths.push(repo_rel);
    }
    paths
}

pub(super) fn extract_ts_string_union(source: &str, target: &str) -> BTreeSet<String> {
    let source = strip_comments(source);
    let pattern = format!(r#"\btype\s+{}\s*=\s*"#, regex::escape(target));
    let Some(start) = Regex::new(&pattern)
        .ok()
        .and_then(|regex| ts_type_alias_body_start(&source, &regex))
    else {
        return BTreeSet::new();
    };
    quoted_strings(&strip_comments(ts_union::body(&source[start..])))
}

fn ts_type_alias_body_start(source: &str, regex: &Regex) -> Option<usize> {
    let mut quote = None;
    let mut escaped = false;
    for (idx, ch) in source.char_indices() {
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active_quote {
                quote = None;
            }
            continue;
        }
        if ch == '"' || ch == '\'' || ch == '`' {
            quote = Some(ch);
            continue;
        }
        if let Some(mat) = regex.find(&source[idx..]).filter(|mat| mat.start() == 0) {
            return Some(idx + mat.end());
        }
    }
    None
}

pub(super) fn extract_ts_const_object_keys(source: &str, target: &str) -> BTreeSet<String> {
    let Some(body) = const_object_body(source, target) else {
        return BTreeSet::new();
    };
    top_level_object_keys(&body)
}

pub(super) fn extract_ts_const_object_property(
    source: &str,
    target: &str,
    property: &str,
) -> BTreeSet<String> {
    let Some(body) = const_object_body(source, target) else {
        return BTreeSet::new();
    };
    top_level_property_values(&body, property)
}

pub(super) fn extract_sql_enum(source: &str, target: &str) -> BTreeSet<String> {
    let source = strip_sql_comments(source);
    let pattern = format!(
        r#"(?is)CREATE\s+TYPE\s+{}\s+AS\s+ENUM\s*\(([^;]+)\)"#,
        regex::escape(target)
    );
    capture_first(&source, &pattern)
        .map(|body| quoted_strings(&body))
        .unwrap_or_default()
}

fn capture_first(source: &str, pattern: &str) -> Option<String> {
    Regex::new(pattern)
        .ok()?
        .captures(source)?
        .get(1)
        .map(|capture| capture.as_str().to_string())
}

trait EmptyStringExt {
    fn if_empty(self, fallback: String) -> String;
}

impl EmptyStringExt for String {
    fn if_empty(self, fallback: String) -> String {
        if self.is_empty() {
            fallback
        } else {
            self
        }
    }
}
