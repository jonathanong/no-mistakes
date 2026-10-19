use serde_yaml::{Mapping, Value};

/// Scopes whose `write` value lets `GITHUB_TOKEN` change GitHub state.
/// `id-token` is intentionally absent: that write is the OIDC token.
const WRITE_SCOPES: &[&str] = &[
    "actions",
    "artifact-metadata",
    "attestations",
    "checks",
    "code-quality",
    "contents",
    "copilot-requests",
    "deployments",
    "discussions",
    "issues",
    "packages",
    "pages",
    "pull-requests",
    "security-events",
    "statuses",
];

/// Documented permission keys. Anything else is not proof of a grant.
const KNOWN_SCOPES: &[&str] = &[
    "actions",
    "artifact-metadata",
    "attestations",
    "checks",
    "code-quality",
    "contents",
    "copilot-requests",
    "deployments",
    "discussions",
    "id-token",
    "issues",
    "packages",
    "pages",
    "pull-requests",
    "security-events",
    "statuses",
    "vulnerability-alerts",
];

pub(super) enum Proof {
    Write(String),
    NotWrite,
    Unknown,
}

enum Origin {
    Job,
    Workflow,
}

enum Access {
    Write,
    Read,
    None,
    Unknown,
}

pub(super) fn triggers_workflow_call(doc: &Mapping) -> bool {
    mapping_get(doc, "on").is_some_and(is_workflow_call)
}

pub(super) fn job_proof(
    job: &Mapping,
    workflow: Option<&Value>,
    callable: bool,
    assume_unspecified: bool,
) -> Proof {
    let specified = if let Some(permissions) = mapping_get(job, "permissions") {
        Some(proof_of(permissions, Origin::Job))
    } else {
        workflow.map(|permissions| proof_of(permissions, Origin::Workflow))
    };
    // No permissions key at either level. Only the opt-in treats that as write.
    // An explicit grant on workflow_call is still not proof below.
    let Some(proof) = specified else {
        return if assume_unspecified {
            Proof::Write(
                "permissions are unspecified and assumeWriteWhenUnspecified is set".to_string(),
            )
        } else {
            Proof::Unknown
        };
    };
    // A called workflow can only keep or lower the caller token. A write
    // grant in this file does not prove the token that actually runs.
    if callable && matches!(proof, Proof::Write(_)) {
        Proof::Unknown
    } else {
        proof
    }
}

fn proof_of(value: &Value, origin: Origin) -> Proof {
    match value {
        Value::String(text) => shorthand(text, origin),
        Value::Mapping(map) if map.is_empty() => Proof::NotWrite,
        Value::Mapping(map) => mapping_proof(map, origin),
        _ => Proof::Unknown,
    }
}

fn shorthand(text: &str, origin: Origin) -> Proof {
    match text.trim() {
        "write-all" => Proof::Write(grant(origin, "write-all")),
        "read-all" => Proof::NotWrite,
        _ => Proof::Unknown,
    }
}

fn mapping_proof(map: &Mapping, origin: Origin) -> Proof {
    let mut unknown = false;
    let mut grant_text = None;
    for (key, value) in map {
        if let Some(text) = judge_scope(key, value) {
            if grant_text.is_none() {
                grant_text = Some(text);
            }
        } else if scope_unknown(key, value) {
            unknown = true;
        }
    }
    if let Some(text) = grant_text {
        Proof::Write(grant(origin, &text))
    } else if unknown {
        Proof::Unknown
    } else {
        Proof::NotWrite
    }
}

fn judge_scope(key: &Value, value: &Value) -> Option<String> {
    let name = key.as_str()?;
    if name != "id-token" && WRITE_SCOPES.contains(&name) && matches!(access(value), Access::Write)
    {
        return Some(format!("{name}: write"));
    }
    None
}

fn scope_unknown(key: &Value, value: &Value) -> bool {
    let Some(name) = key.as_str() else {
        return true;
    };
    if !KNOWN_SCOPES.contains(&name) {
        return true;
    }
    match access(value) {
        Access::Read | Access::None => false,
        // `id-token: write` is the OIDC token. Other known `write` values that
        // are not in `WRITE_SCOPES` (only `vulnerability-alerts`) are not proof.
        Access::Write => name != "id-token",
        Access::Unknown => true,
    }
}

fn access(value: &Value) -> Access {
    match value.as_str().map(str::trim) {
        Some("write") => Access::Write,
        Some("read") => Access::Read,
        Some("none") => Access::None,
        _ => Access::Unknown,
    }
}

fn grant(origin: Origin, text: &str) -> String {
    match origin {
        Origin::Job => format!("job permissions grant {text}"),
        Origin::Workflow => format!("workflow permissions grant {text}"),
    }
}

fn is_workflow_call(on: &Value) -> bool {
    match on {
        Value::String(name) => name == "workflow_call",
        Value::Sequence(items) => items
            .iter()
            .any(|item| item.as_str() == Some("workflow_call")),
        Value::Mapping(map) => map.keys().any(|key| key.as_str() == Some("workflow_call")),
        _ => false,
    }
}

fn mapping_get<'a>(map: &'a Mapping, key: &str) -> Option<&'a Value> {
    map.get(Value::String(key.to_owned()))
}
