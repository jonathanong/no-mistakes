use super::{CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::ts_source::relative_slash_path;
use regex::Regex;
use std::collections::BTreeSet;
use std::path::Path;
use std::sync::LazyLock;

static YAML_FRAGMENT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:^|[^A-Za-z0-9_-])timeout-minutes:\s*['"]?\d+['"]?"#).expect("yaml fragment")
});
static TIMEOUT_PROPERTY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\[['"]timeout-minutes['"]\]|\.timeoutMinutes\b"#).expect("property")
});
static LITERAL_EQUALITY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\)\.(?:toBe|toEqual)\(\s*(?:['"]\d+['"]|\d+)"#).expect("equality")
});
static CONTAIN_DIGIT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\)\.toContain\([^)]*\d[^)]*\)").expect("contain"));

pub(super) fn check_file(
    root: &Path,
    path: &Path,
    opts: &CompiledOptions,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<RuleFinding> {
    let rel = relative_slash_path(root, path);
    let Some(source) = super::super::read_source(sources, path) else {
        return Vec::new();
    };
    check_source(&rel, &source, opts)
}

pub(super) fn check_source(rel: &str, source: &str, opts: &CompiledOptions) -> Vec<RuleFinding> {
    if crate::codebase::ts_source::has_disable_file_comment(source, RULE_ID) {
        return Vec::new();
    }
    let mut findings = Vec::new();
    let mut used_allow = BTreeSet::new();
    for (index, line) in source.lines().enumerate() {
        findings.extend(line_findings(rel, index + 1, line, opts, &mut used_allow));
    }
    findings.extend(stale_allow_findings(rel, opts, &used_allow));
    super::super::suppress_rule_findings_with_source(&mut findings, source);
    findings
}

fn line_findings(
    rel: &str,
    line_no: usize,
    line: &str,
    opts: &CompiledOptions,
    used_allow: &mut BTreeSet<String>,
) -> Vec<RuleFinding> {
    if !is_violation(line) {
        return Vec::new();
    }
    let trimmed = line.trim();
    let key = format!("{rel}#{trimmed}");
    if let Some(reason) = opts.allow.get(&key) {
        used_allow.insert(key.clone());
        if reason.trim().is_empty() {
            return vec![finding(
                rel,
                line_no,
                format!("{rel}: allow entry `{key}` has no reason"),
                &key,
            )];
        }
        return Vec::new();
    }
    vec![finding(
        rel,
        line_no,
        format!(
            "{rel}: duplicates a timeout-minutes value; delete the assertion or add a reasoned allow entry"
        ),
        trimmed,
    )]
}

fn stale_allow_findings(
    rel: &str,
    opts: &CompiledOptions,
    used_allow: &BTreeSet<String>,
) -> Vec<RuleFinding> {
    let prefix = format!("{rel}#");
    opts.allow
        .keys()
        .filter(|allowed| !used_allow.contains(*allowed) && allowed.starts_with(&prefix))
        .map(|allowed| {
            finding(
                rel,
                1,
                format!("stale github-actions-test-timeout-literals allow entry `{allowed}`"),
                allowed,
            )
        })
        .collect()
}

fn is_violation(line: &str) -> bool {
    if line.trim_start().starts_with("//") {
        return false;
    }
    YAML_FRAGMENT.is_match(line)
        || (TIMEOUT_PROPERTY.is_match(line)
            && (LITERAL_EQUALITY.is_match(line) || CONTAIN_DIGIT.is_match(line)))
}

fn finding(file: &str, line: usize, message: String, target: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message,
        import: None,
        target: Some(target.to_string()),
    }
}
