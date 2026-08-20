use super::{AssertionKind, MatchMode, ValueAssertion, RULE_ID};
use crate::codebase::rules::RuleFinding;
use anyhow::Result;
use serde_yaml::Value;

mod kinds;
mod selector;
use kinds::kind_violation;
use selector::{any_groups, values_at_selector};

pub(super) fn assert_value(
    rel: &str,
    root: &Value,
    assertion: &ValueAssertion,
) -> Result<Vec<RuleFinding>> {
    let Some(kind) = assertion.kind else {
        return Ok(Vec::new());
    };
    if assertion.key.is_empty() || kind == AssertionKind::EqualsFile {
        return Ok(Vec::new());
    }
    if assertion.match_mode == MatchMode::Any {
        return Ok(assert_any(rel, root, assertion, kind));
    }
    let selected = values_at_selector(root, &assertion.key);
    let mut findings = Vec::new();
    if selected.has_missing {
        findings.push(assertion_finding(
            rel,
            assertion,
            format!(
                "{rel}: config value `{}` required by assertion is missing",
                assertion.key
            ),
        ));
    }
    if selected.values.is_empty() {
        return Ok(findings);
    }
    for value in selected.values {
        if let Some(reason) = kind_violation(value, assertion, kind) {
            findings.push(assertion_finding(
                rel,
                assertion,
                format!("{rel}: config value `{}` {reason}", assertion.key),
            ));
        }
    }
    Ok(findings)
}

fn assert_any(
    rel: &str,
    root: &Value,
    assertion: &ValueAssertion,
    kind: AssertionKind,
) -> Vec<RuleFinding> {
    let failed = any_groups(root, &assertion.key).into_iter().any(|group| {
        !group
            .iter()
            .any(|value| kind_violation(value, assertion, kind).is_none())
    });
    if !failed {
        return Vec::new();
    }
    vec![assertion_finding(
        rel,
        assertion,
        format!(
            "{rel}: config value `{}` must match at least one array entry",
            assertion.key
        ),
    )]
}

fn assertion_finding(rel: &str, assertion: &ValueAssertion, fallback: String) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: rel.to_string(),
        line: 1,
        message: assertion.message.clone().unwrap_or(fallback),
        import: None,
        target: Some(assertion.key.clone()),
    }
}
