use crate::codebase::keyed_diff::diff_entries;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SwiftResolvedPin {
    pub identity: String,
    pub location: String,
    pub version: String,
    pub revision: String,
    pub checksum: String,
    pub branch: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SwiftResolvedDiagnostic {
    Malformed,
    UnsupportedSchema,
}

pub(crate) fn parse_resolved_pins(
    source: &str,
) -> Result<Vec<SwiftResolvedPin>, SwiftResolvedDiagnostic> {
    let value: Value =
        serde_json::from_str(source).map_err(|_| SwiftResolvedDiagnostic::Malformed)?;
    let version = value
        .get("version")
        .and_then(Value::as_u64)
        .ok_or(SwiftResolvedDiagnostic::UnsupportedSchema)?;
    let (pins, location_field) = match version {
        1 => (
            value
                .get("object")
                .ok_or(SwiftResolvedDiagnostic::UnsupportedSchema)?
                .get("pins"),
            "repositoryURL",
        ),
        2 | 3 => (value.get("pins"), "location"),
        _ => return Err(SwiftResolvedDiagnostic::UnsupportedSchema),
    };
    let pins = pins
        .and_then(Value::as_array)
        .ok_or(SwiftResolvedDiagnostic::Malformed)?;
    let mut result = Vec::new();
    for pin in pins {
        let location = pin
            .get(location_field)
            .and_then(Value::as_str)
            .unwrap_or("");
        let identity = if version == 1 {
            legacy_identity(location)
        } else {
            pin.get("identity")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string()
        };
        if identity.is_empty() {
            return Err(SwiftResolvedDiagnostic::Malformed);
        }
        let state = pin
            .get("state")
            .and_then(Value::as_object)
            .ok_or(SwiftResolvedDiagnostic::Malformed)?;
        result.push(SwiftResolvedPin {
            identity,
            location: location.to_string(),
            version: state
                .get("version")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_string(),
            revision: state
                .get("revision")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_string(),
            branch: state
                .get("branch")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            checksum: state
                .get("checksum")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_string(),
        });
    }
    result.sort();
    result.dedup();
    if result
        .windows(2)
        .any(|pair| pair[0].identity == pair[1].identity)
    {
        return Err(SwiftResolvedDiagnostic::Malformed);
    }
    Ok(result)
}

fn legacy_identity(location: &str) -> String {
    let location = location.trim_end_matches(['/', '\\']);
    let name = location.rsplit(['/', '\\']).next().unwrap_or("");
    name.strip_suffix(".git").unwrap_or(name).to_lowercase()
}

pub(crate) fn diff_resolved_pins(
    before: &[SwiftResolvedPin],
    after: &[SwiftResolvedPin],
) -> Vec<String> {
    let before: BTreeMap<_, _> = before
        .iter()
        .map(|pin| (pin.identity.as_str(), pin))
        .collect();
    let after: BTreeMap<_, _> = after
        .iter()
        .map(|pin| (pin.identity.as_str(), pin))
        .collect();
    let mut ids: Vec<_> = diff_entries(&before, &after).all_ids().collect();
    ids.sort();
    ids
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
#[path = "resolved_tests.rs"]
mod resolved_tests;
