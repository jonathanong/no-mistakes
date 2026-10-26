/// Older policy contracts retain every diagnostic except the newly additive
/// hygiene advisories, which have dedicated aggregate CLI/N-API coverage.
pub(crate) fn without_hygiene_advisories(
    mut report: crate::check_runner::CheckResults,
) -> crate::check_runner::CheckResults {
    report.advisories.retain(|row| !is_hygiene(&row.rule));
    report
}

pub(crate) fn without_hygiene_advisory_json(mut report: serde_json::Value) -> serde_json::Value {
    retain(&mut report);
    report
}

fn is_hygiene(rule: &str) -> bool {
    matches!(
        rule,
        "unused-disable-directive" | "invalid-disable-directive"
    )
}

fn retain(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(object) => {
            if let Some(rows) = object
                .get_mut("advisories")
                .and_then(|value| value.as_array_mut())
            {
                rows.retain(|row| row["rule"].as_str().is_none_or(|rule| !is_hygiene(rule)));
            }
            for value in object.values_mut() {
                retain(value);
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                retain(value);
            }
        }
        _ => {}
    }
}

#[path = "policy_advisories_tests.rs"]
mod tests;
