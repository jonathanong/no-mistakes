use super::{compare, document::Documents, references, schema, DeclaredPayloadContract};
use crate::codebase::ts_source::SourceStore;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::Path;

#[derive(Debug)]
pub(super) enum Outcome {
    Proven,
    Unanalyzable { file: String, reason: String },
}

pub(super) fn check(
    root: &Path,
    contract: &DeclaredPayloadContract,
    allowed: &BTreeSet<String>,
    sources: &SourceStore,
    documents: &Documents,
) -> Result<Outcome, (String, String)> {
    if contract.name.trim().is_empty() {
        return Err((
            contract.producer.file.clone(),
            "unproven: name must be nonempty".into(),
        ));
    }
    let error = |file: &str, side: &str, reason: String| {
        (file.to_owned(), format!("unproven {side}: {reason}"))
    };
    // Load both physical declarations before policy may ignore nested shapes.
    // Invalid consumer JSON or an unavailable pointer remains a hard finding.
    let producer = references::read_document(root, &contract.producer, allowed, sources, documents)
        .map_err(|reason| error(&contract.producer.file, "producer", reason))?;
    let consumer = references::read_document(root, &contract.consumer, allowed, sources, documents)
        .map_err(|reason| error(&contract.consumer.file, "consumer", reason))?;
    let producer = references::selected(&producer, &contract.producer)
        .map_err(|reason| error(&contract.producer.file, "producer", reason))?;
    let consumer = references::selected(&consumer, &contract.consumer)
        .map_err(|reason| error(&contract.consumer.file, "consumer", reason))?;
    // Inspect immediate schema children only, before recursive normalization or
    // compatibility proof. This is a grammar boundary, not a validity claim.
    for (side, reference, value) in [
        ("producer", &contract.producer, producer),
        ("consumer", &contract.consumer, consumer),
    ] {
        if let Some(path) = nested_child(value) {
            return Ok(Outcome::Unanalyzable {
                file: reference.file.clone(),
                reason: format!(
                "unproven: {side} schema `{}{}` at {path} is unanalyzable at effort low; low supports scalar roots, flat objects with scalar properties, and root arrays with scalar items. Use effort max, or flatten both declarations and their actual payloads; selecting leaf pointers alone does not preserve parent constraints",
                reference.file, reference.pointer,
            ),
            });
        }
    }
    let producer = schema::parse(producer, "$", 0)
        .map_err(|reason| error(&contract.producer.file, "producer", reason))?;
    let consumer = schema::parse(consumer, "$", 0)
        .map_err(|reason| error(&contract.consumer.file, "consumer", reason))?;
    compare::subset(&producer, &consumer, "$").map_err(|reason| {
        (
            contract.producer.file.clone(),
            format!(
                "incompatible with {}{}: {reason}",
                contract.consumer.file, contract.consumer.pointer
            ),
        )
    })?;
    Ok(Outcome::Proven)
}

fn nested_child(value: &Value) -> Option<String> {
    match value.get("type").and_then(Value::as_str) {
        Some("object") => {
            value
                .get("properties")
                .and_then(Value::as_object)
                .and_then(|properties| {
                    properties
                        .iter()
                        .find(|(_, child)| container(child))
                        .map(|(name, _)| format!("$[{name:?}]"))
                })
        }
        Some("array") if value.get("items").is_some_and(container) => Some("$[]".into()),
        _ => None,
    }
}

fn container(value: &Value) -> bool {
    matches!(
        value.get("type").and_then(Value::as_str),
        Some("object" | "array")
    )
}
