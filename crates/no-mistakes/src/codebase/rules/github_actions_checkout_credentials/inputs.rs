use super::permissions::Proof;
use serde_yaml::{Mapping, Value};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Persist {
    IsFalse,
    NotFalse,
    Unknown,
}

pub(super) enum Verdict {
    Safe,
    Write(String),
    Unknown(&'static str),
}

pub(super) fn verdict(proof: &Proof, step: &Mapping) -> Verdict {
    match persist(step) {
        Persist::IsFalse => return Verdict::Safe,
        Persist::Unknown => {
            return Verdict::Unknown("nonliteral persist-credentials or non-mapping with")
        }
        Persist::NotFalse => {}
    }
    // A custom token's identity and permissions cannot be proven from this workflow,
    // even when its expression mentions secrets.GITHUB_TOKEN.
    if mapping_get(step, "with")
        .and_then(Value::as_mapping)
        .is_some_and(|map| mapping_get(map, "token").is_some())
    {
        return Verdict::Unknown("custom with.token has unknown identity or permissions");
    }
    match proof {
        Proof::Write(reason) => Verdict::Write(reason.clone()),
        Proof::NotWrite => Verdict::Safe,
        Proof::Unknown => Verdict::Unknown("permissions are unknown or unspecified"),
    }
}

fn persist(step: &Mapping) -> Persist {
    let Some(with) = mapping_get(step, "with") else {
        return Persist::NotFalse;
    };
    match with {
        Value::Null => Persist::NotFalse,
        Value::Mapping(map) => match mapping_get(map, "persist-credentials") {
            None => Persist::NotFalse,
            Some(value) => classify_persist(value),
        },
        _ => Persist::Unknown,
    }
}

fn classify_persist(value: &Value) -> Persist {
    match value {
        Value::Bool(false) => Persist::IsFalse,
        Value::Bool(true) => Persist::NotFalse,
        Value::String(text) => classify_persist_str(text),
        _ => Persist::Unknown,
    }
}

fn classify_persist_str(text: &str) -> Persist {
    match text.trim().to_ascii_lowercase().as_str() {
        "true" => Persist::NotFalse,
        "false" => Persist::IsFalse,
        _ => Persist::Unknown,
    }
}

fn mapping_get<'a>(map: &'a Mapping, key: &str) -> Option<&'a Value> {
    map.get(Value::String(key.to_owned()))
}
