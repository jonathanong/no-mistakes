use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use regex::Regex;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "finite-set-consistency";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) sets: Vec<SetSpec>,
    pub(crate) comparisons: Vec<Comparison>,
}

#[derive(Clone, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct SetSpec {
    pub(crate) name: String,
    pub(crate) file: String,
    pub(crate) kind: String,
    pub(crate) target: String,
    pub(crate) property: String,
    pub(crate) pattern: String,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Comparison {
    pub(crate) left: String,
    pub(crate) right: String,
    pub(crate) message: Option<String>,
}

#[derive(Debug, Clone)]
struct ExtractedSet {
    file: String,
    values: BTreeSet<String>,
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| -> Result<Vec<RuleFinding>> {
            let opts: Options = rule.rule_options();
            let target_roots = super::target_roots(root, config, rule);
            let skip = super::skip_dir_set(config);
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|p| super::file_allowed_by_roots_and_skip(root, &skip, p, &target_roots))
                .cloned()
                .collect();
            let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
            scan(root, &opts, &files)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn scan(root: &Path, opts: &Options, files: &[PathBuf]) -> Result<Vec<RuleFinding>> {
    let mut sets = BTreeMap::new();
    for spec in &opts.sets {
        if spec.name.is_empty() {
            continue;
        }
        sets.insert(spec.name.clone(), extract_set(root, spec, files)?);
    }

    let mut findings = Vec::new();
    for comparison in &opts.comparisons {
        let (Some(left), Some(right)) = (sets.get(&comparison.left), sets.get(&comparison.right))
        else {
            continue;
        };
        for value in left.values.difference(&right.values) {
            findings.push(finding(
                &right.file,
                comparison,
                format!(
                    "{} contains `{}` but {} does not",
                    comparison.left, value, comparison.right
                ),
                value,
            ));
        }
        for value in right.values.difference(&left.values) {
            findings.push(finding(
                &left.file,
                comparison,
                format!(
                    "{} contains `{}` but {} does not",
                    comparison.right, value, comparison.left
                ),
                value,
            ));
        }
    }
    findings.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
    Ok(findings)
}

fn finding(file: &str, comparison: &Comparison, fallback: String, value: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line: 1,
        message: comparison.message.clone().unwrap_or(fallback),
        import: None,
        target: Some(value.to_string()),
    }
}

fn extract_set(root: &Path, spec: &SetSpec, files: &[PathBuf]) -> Result<ExtractedSet> {
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

fn extract_path_regex_set(root: &Path, spec: &SetSpec, files: &[PathBuf]) -> Result<ExtractedSet> {
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

fn extract_ts_string_union(source: &str, target: &str) -> BTreeSet<String> {
    let pattern = format!(r#"(?s)\btype\s+{}\s*=\s*([^;]+);"#, regex::escape(target));
    let Some(body) = capture_first(source, &pattern) else {
        return BTreeSet::new();
    };
    quoted_strings(&body)
}

fn extract_ts_const_object_keys(source: &str, target: &str) -> BTreeSet<String> {
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

fn extract_ts_const_object_property(
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

fn extract_sql_enum(source: &str, target: &str) -> BTreeSet<String> {
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

fn matching_brace(source: &str, open: usize) -> Option<usize> {
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

#[cfg(test)]
mod tests;
