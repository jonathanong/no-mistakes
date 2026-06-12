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
    let pattern = format!(r#"(?s)\btype\s+{}\s*=\s*([^;]+);"#, regex::escape(target));
    let Some(body) = capture_first(source, &pattern) else {
        return BTreeSet::new();
    };
    quoted_strings(&body)
}

pub(super) fn extract_ts_const_object_keys(source: &str, target: &str) -> BTreeSet<String> {
    let Some(body) = const_object_body(source, target) else {
        return BTreeSet::new();
    };
    let regex = Regex::new(r#"(?m)(?:^|[,{])\s*(?:"([^"]+)"|'([^']+)'|([A-Za-z_$][\w$-]*))\s*:"#)
        .expect("object key regex compiles");
    regex
        .captures_iter(&body)
        .filter_map(|captures| {
            captures
                .get(1)
                .or_else(|| captures.get(2))
                .or_else(|| captures.get(3))
                .map(|capture| capture.as_str().to_string())
        })
        .collect()
}

pub(super) fn extract_ts_const_object_property(
    source: &str,
    target: &str,
    property: &str,
) -> BTreeSet<String> {
    let Some(body) = const_object_body(source, target) else {
        return BTreeSet::new();
    };
    let pattern = format!(
        r#"{}\s*:\s*(?:"([^"]+)"|'([^']+)')"#,
        regex::escape(property)
    );
    let regex = Regex::new(&pattern).expect("object property regex compiles");
    regex
        .captures_iter(&body)
        .filter_map(|captures| {
            captures
                .get(1)
                .or_else(|| captures.get(2))
                .map(|capture| capture.as_str().to_string())
        })
        .collect()
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

fn const_object_body(source: &str, target: &str) -> Option<String> {
    let pattern = format!(r#"\bconst\s+{}\s*="#, regex::escape(target));
    let mat = Regex::new(&pattern).ok()?.find(source)?;
    let open = source[mat.end()..].find('{')? + mat.end();
    let close = matching_brace(source, open)?;
    source.get(open + 1..close).map(str::to_string)
}

fn capture_first(source: &str, pattern: &str) -> Option<String> {
    Regex::new(pattern)
        .ok()?
        .captures(source)?
        .get(1)
        .map(|capture| capture.as_str().to_string())
}

pub(super) fn matching_brace(source: &str, open: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut quote = None;
    let mut escaped = false;
    for (idx, ch) in source.char_indices().skip_while(|(idx, _)| *idx < open) {
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
        match ch {
            '"' | '\'' | '`' => quote = Some(ch),
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(idx);
                }
            }
            _ => {}
        }
    }
    None
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
