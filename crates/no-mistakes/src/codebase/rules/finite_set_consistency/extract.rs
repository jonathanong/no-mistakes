use super::comments::{strip_comments, strip_sql_comments};
use super::literals::{quoted_strings_sql, quoted_strings_ts};
pub(super) use super::markdown::extract_markdown_table_code_cells;
use super::object::{const_object_body, top_level_object_keys, top_level_property_values};
pub(super) use super::ts_array::{extract_ts_array_literal, extract_ts_const_array_property};
use super::ts_union;
pub(super) use super::yaml::extract_yaml_sequence;
use super::SetSpec;
use crate::codebase::ts_source::relative_slash_path;
use anyhow::Result;
use regex::Regex;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

mod path;
pub(super) use path::extract_path_regex_set;

#[derive(Debug, Clone)]
pub(super) struct ExtractedSet {
    pub(super) file: String,
    pub(super) values: BTreeSet<String>,
}

pub(super) fn extract_set_with_sources(
    root: &Path,
    spec: &SetSpec,
    files: &[PathBuf],
    target_roots: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<ExtractedSet> {
    if spec.kind == "path-regex-capture" {
        return extract_path_regex_set(root, spec, files, target_roots);
    }
    let paths = resolve_spec_files(root, &spec.file, target_roots);
    let mut values = BTreeSet::new();
    for path in &paths {
        let source = sources
            .read_path(path)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        values.extend(match spec.kind.as_str() {
            "ts-string-union" => extract_ts_string_union(&source, &spec.target),
            "ts-const-object-keys" => extract_ts_const_object_keys(&source, &spec.target),
            "ts-const-object-property" => {
                extract_ts_const_object_property(&source, &spec.target, &spec.property)
            }
            "ts-array-literal" => extract_ts_array_literal(&source, &spec.target),
            "ts-const-array-property" => {
                extract_ts_const_array_property(&source, &spec.target, &spec.property)
            }
            "yaml-sequence" => extract_yaml_sequence(&source, &spec.key),
            "markdown-table-code-cells" => extract_markdown_table_code_cells(&source),
            "sql-enum" => extract_sql_enum(&source, &spec.target),
            _ => BTreeSet::new(),
        });
    }
    let path = paths
        .first()
        .expect("resolve_spec_files always returns at least one path");
    Ok(ExtractedSet {
        file: relative_slash_path(root, path),
        values,
    })
}

fn resolve_spec_files(root: &Path, file: &str, target_roots: &[PathBuf]) -> Vec<PathBuf> {
    let repo_path = root.join(file);
    if repo_path.exists() {
        return vec![repo_path];
    }
    let paths = target_roots
        .iter()
        .filter(|target_root| *target_root != root)
        .map(|target_root| target_root.join(file))
        .filter(|path| path.exists())
        .collect::<Vec<_>>();
    if paths.is_empty() {
        vec![repo_path]
    } else {
        paths
    }
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
    quoted_strings_ts(&strip_comments(ts_union::body(&source[start..])))
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
        r#"(?is)CREATE\s+TYPE\s+{}\s+AS\s+ENUM\s*\("#,
        regex::escape(target)
    );
    Regex::new(&pattern)
        .ok()
        .and_then(|regex| regex.find(&source))
        .and_then(|mat| sql_enum_body(&source[mat.end()..]))
        .map(quoted_strings_sql)
        .unwrap_or_default()
}

fn sql_enum_body(source: &str) -> Option<&str> {
    let mut quote = false;
    let mut chars = source.char_indices().peekable();
    while let Some((idx, ch)) = chars.next() {
        if quote {
            if ch == '\'' {
                if chars.peek().is_some_and(|(_, next)| *next == '\'') {
                    chars.next();
                } else {
                    quote = false;
                }
            }
            continue;
        }
        if ch == '\'' {
            quote = true;
            continue;
        }
        if ch == ')' {
            return Some(&source[..idx]);
        }
    }
    None
}
