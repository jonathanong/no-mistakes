/// Policy tests inspect their own rule's channel. Suppression hygiene remains
/// computed and is covered by the dedicated aggregate CLI/N-API contracts.
pub(crate) fn policy_advisories(
    mut report: crate::check_runner::CheckResults,
    rule_prefix: &str,
) -> crate::check_runner::CheckResults {
    report
        .advisories
        .retain(|row| row.rule.starts_with(rule_prefix));
    report
}

pub(crate) fn policy_advisory_json(
    mut report: serde_json::Value,
    rule_prefix: &str,
) -> serde_json::Value {
    retain(&mut report, rule_prefix);
    report
}

fn retain(value: &mut serde_json::Value, rule_prefix: &str) {
    match value {
        serde_json::Value::Object(object) => {
            if let Some(rows) = object
                .get_mut("advisories")
                .and_then(|value| value.as_array_mut())
            {
                rows.retain(|row| {
                    row["rule"]
                        .as_str()
                        .is_some_and(|rule| rule.starts_with(rule_prefix))
                });
            }
            for value in object.values_mut() {
                retain(value, rule_prefix);
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                retain(value, rule_prefix);
            }
        }
        _ => {}
    }
}
