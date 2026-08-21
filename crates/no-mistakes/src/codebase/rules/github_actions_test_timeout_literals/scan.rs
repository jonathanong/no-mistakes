use super::{CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::ts_source::relative_slash_path;
use regex::Regex;
use std::collections::BTreeSet;
use std::path::Path;
use std::sync::LazyLock;

static YAML_FRAGMENT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?:^|[^A-Za-z0-9_-])(?:'timeout-minutes'|"timeout-minutes"|timeout-minutes):\s*(?:'\d+'|"\d+"|\d+\b)"#,
    )
    .expect("yaml fragment")
});
static TIMEOUT_EQUALITY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"expect\([^;]*?(?:\[['"]timeout-minutes['"]\]|\.timeoutMinutes\b)[^;]*?\)(?:\.not)?\.(?:toBe|toEqual|toStrictEqual)\(\s*(?:['"]\d+['"]|\d+)"#,
    )
    .expect("equality")
});
static TIMEOUT_CONTAIN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"expect\([^;]*?(?:\[['"]timeout-minutes['"]\]|\.timeoutMinutes\b)[^;]*?\)\.toContain\([^)]*\d[^)]*\)"#,
    )
    .expect("contain")
});

pub(super) fn check_file(
    root: &Path,
    path: &Path,
    opts: &CompiledOptions,
    sources: &crate::codebase::ts_source::SourceStore,
    defer_suppression: bool,
) -> Vec<RuleFinding> {
    let rel = relative_slash_path(root, path);
    let Some(source) = super::super::read_source(sources, path) else {
        return Vec::new();
    };
    check_source(&rel, &source, opts, defer_suppression)
}

pub(super) fn check_source(
    rel: &str,
    source: &str,
    opts: &CompiledOptions,
    defer_suppression: bool,
) -> Vec<RuleFinding> {
    if !defer_suppression && crate::codebase::ts_source::has_disable_file_comment(source, RULE_ID) {
        return Vec::new();
    }
    let mut findings = Vec::new();
    let mut used_allow = BTreeSet::new();
    for (index, line) in source.lines().enumerate() {
        findings.extend(line_findings(rel, index + 1, line, opts, &mut used_allow));
    }
    findings.extend(stale_allow_findings(rel, opts, &used_allow));
    if !defer_suppression {
        super::super::suppress_rule_findings_with_source(&mut findings, source);
    }
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
        || TIMEOUT_EQUALITY.is_match(line)
        || TIMEOUT_CONTAIN.is_match(line)
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
