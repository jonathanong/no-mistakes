use serde_yaml::Value;

#[derive(Default)]
pub(super) struct Summary {
    pub(super) writes: Vec<String>,
    pub(super) unresolved: bool,
}

pub(super) fn inspect(value: &Value) -> Summary {
    let mut result = Summary::default();
    match value {
        Value::String(text) if text == "write-all" => result.writes.push("write-all".into()),
        Value::String(text) if text == "read-all" => {}
        Value::Mapping(map) => {
            for (key, value) in map {
                let Some(scope) = key.as_str() else {
                    result.unresolved = true;
                    continue;
                };
                if scope == "<<" {
                    result.unresolved = true;
                    continue;
                }
                if !valid_scope(scope) {
                    result.unresolved = true;
                    continue;
                }
                match value.as_str() {
                    Some("write") => result.writes.push(format!("{scope}: write")),
                    Some("read" | "none") => {}
                    _ => result.unresolved = true,
                }
            }
        }
        _ => result.unresolved = true,
    }
    result
}

fn valid_scope(scope: &str) -> bool {
    matches!(
        scope,
        "actions"
            | "artifact-metadata"
            | "attestations"
            | "checks"
            | "code-quality"
            | "contents"
            | "copilot-requests"
            | "deployments"
            | "discussions"
            | "id-token"
            | "issues"
            | "packages"
            | "pages"
            | "pull-requests"
            | "security-events"
            | "statuses"
            | "vulnerability-alerts"
    )
}
