use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SwiftResolvedPin {
    pub identity: String,
    pub location: String,
    pub version: String,
    pub revision: String,
    pub checksum: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SwiftResolvedDiagnostic {
    Malformed,
    UnsupportedSchema,
}

pub(crate) fn parse_resolved_pins(
    source: &str,
) -> Result<Vec<SwiftResolvedPin>, SwiftResolvedDiagnostic> {
    let value: serde_json::Value =
        serde_json::from_str(source).map_err(|_| SwiftResolvedDiagnostic::Malformed)?;
    let version = value
        .get("version")
        .and_then(serde_json::Value::as_u64)
        .ok_or(SwiftResolvedDiagnostic::UnsupportedSchema)?;
    if !matches!(version, 2 | 3) {
        return Err(SwiftResolvedDiagnostic::UnsupportedSchema);
    }
    let pins = value
        .get("pins")
        .and_then(serde_json::Value::as_array)
        .ok_or(SwiftResolvedDiagnostic::Malformed)?;
    let mut result = Vec::new();
    for pin in pins {
        let identity = pin
            .get("identity")
            .and_then(serde_json::Value::as_str)
            .ok_or(SwiftResolvedDiagnostic::Malformed)?;
        let location = pin
            .get("location")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let state = pin
            .get("state")
            .and_then(serde_json::Value::as_object)
            .ok_or(SwiftResolvedDiagnostic::Malformed)?;
        result.push(SwiftResolvedPin {
            identity: identity.to_string(),
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
            checksum: state
                .get("checksum")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_string(),
        });
    }
    result.sort();
    result.dedup();
    Ok(result)
}

pub(crate) fn diff_resolved_pins(
    before: &[SwiftResolvedPin],
    after: &[SwiftResolvedPin],
) -> Vec<String> {
    let before: BTreeMap<_, _> = before
        .iter()
        .map(|pin| (pin.identity.clone(), pin))
        .collect();
    let after: BTreeMap<_, _> = after
        .iter()
        .map(|pin| (pin.identity.clone(), pin))
        .collect();
    before
        .keys()
        .chain(after.keys())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|identity| before.get(*identity) != after.get(*identity))
        .cloned()
        .collect()
}

#[cfg(test)]
#[path = "resolved_tests.rs"]
mod resolved_tests;
