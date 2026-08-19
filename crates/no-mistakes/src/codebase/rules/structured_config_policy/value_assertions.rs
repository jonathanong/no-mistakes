use super::{AssertionKind, MatchMode, ValueAssertion, RULE_ID};
use crate::codebase::rules::RuleFinding;
use anyhow::Result;
use serde_yaml::Value;

mod kinds;
use kinds::kind_violation;

pub(super) fn assert_value(
    rel: &str,
    root: &Value,
    assertion: &ValueAssertion,
) -> Result<Vec<RuleFinding>> {
    let Some(kind) = assertion.kind else {
        return Ok(Vec::new());
    };
    if assertion.key.is_empty() {
        return Ok(Vec::new());
    }
    let selected = values_at_selector(root, &assertion.key);
    if assertion.match_mode == MatchMode::Any {
        return Ok(assert_any(rel, assertion, kind, &selected));
    }
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
    assertion: &ValueAssertion,
    kind: AssertionKind,
    selected: &SelectorValues<'_>,
) -> Vec<RuleFinding> {
    if selected
        .values
        .iter()
        .any(|value| kind_violation(value, assertion, kind).is_none())
    {
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

struct SelectorValues<'a> {
    values: Vec<&'a Value>,
    has_missing: bool,
}

fn values_at_selector<'a>(value: &'a Value, selector: &str) -> SelectorValues<'a> {
    let mut current = vec![Some(value)];
    let mut has_missing = false;
    for part in selector.split('.') {
        let mut next = Vec::new();
        if part == "[]" {
            for value in current {
                match value {
                    Some(Value::Sequence(items)) => next.extend(items.iter().map(Some)),
                    Some(_) | None => {
                        has_missing = true;
                        next.push(None);
                    }
                }
            }
        } else if let Ok(index) = part.parse::<usize>() {
            for value in current {
                match value {
                    Some(Value::Sequence(items)) => match items.get(index) {
                        Some(item) => next.push(Some(item)),
                        None => {
                            has_missing = true;
                            next.push(None);
                        }
                    },
                    Some(_) | None => {
                        has_missing = true;
                        next.push(None);
                    }
                }
            }
        } else {
            for value in current {
                match value.and_then(|value| value.get(part)) {
                    Some(child) => next.push(Some(child)),
                    None => {
                        has_missing = true;
                        next.push(None);
                    }
                }
            }
        }
        current = next;
    }
    SelectorValues {
        values: current.into_iter().flatten().collect(),
        has_missing,
    }
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
