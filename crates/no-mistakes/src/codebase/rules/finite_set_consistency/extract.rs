use super::object::{const_object_body, top_level_object_keys, top_level_property_values};
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

pub(super) fn extract_set(root: &Path, spec: &SetSpec, files: &[PathBuf]) -> Result<ExtractedSet> {
    if spec.kind == "path-regex-capture" {
        return extract_path_regex_set(root, spec, files);
    }
    let source = std::fs::read_to_string(root.join(&spec.file))?;
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
        file: spec.file.clone(),
        values,
    })
}

pub(super) fn extract_path_regex_set(
    root: &Path,
    spec: &SetSpec,
    files: &[PathBuf],
) -> Result<ExtractedSet> {
    let regex = Regex::new(&spec.pattern)?;
    let mut values = BTreeSet::new();
    for file in files {
        let rel = relative_slash_path(root, file);
        let Some(captures) = regex.captures(&rel) else {
            continue;
        };
        let value = captures
            .name("value")
            .or_else(|| captures.get(1))
            .map(|capture| capture.as_str().to_string());
        values.extend(value);
    }
    Ok(ExtractedSet {
        file: spec.file.clone().if_empty(".".to_string()),
        values,
    })
}

pub(super) fn extract_ts_string_union(source: &str, target: &str) -> BTreeSet<String> {
    let pattern = format!(
        r#"(?s)\btype\s+{}\s*=\s*(.*?)(?:;|\n\s*\n|$)"#,
        regex::escape(target)
    );
    let Some(body) = capture_first(source, &pattern) else {
        return BTreeSet::new();
    };
    quoted_strings(&body)
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
    let pattern = format!(
        r#"(?is)CREATE\s+TYPE\s+{}\s+AS\s+ENUM\s*\(([^;]+)\)"#,
        regex::escape(target)
    );
    capture_first(source, &pattern)
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

fn quoted_strings(source: &str) -> BTreeSet<String> {
    let regex = Regex::new(r#""([^"]+)"|'([^']+)'"#).expect("quoted string regex compiles");
    regex
        .captures_iter(source)
        .filter_map(|captures| {
            captures
                .get(1)
                .or_else(|| captures.get(2))
                .map(|capture| capture.as_str().to_string())
        })
        .collect()
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
