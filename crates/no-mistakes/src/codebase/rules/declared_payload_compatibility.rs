//! Directional compatibility of explicitly selected JSON Schema declarations.
//! This checks declarations, not whether runtime endpoints adopt those schemas.
use super::RuleFinding;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

mod compare;
mod document;
mod effort;
mod low;
mod numeric;
mod references;
mod schema;

#[cfg(test)]
#[path = "declared_payload_compatibility/probe_tests.rs"]
mod probe;
#[cfg(not(test))]
#[path = "declared_payload_compatibility/probe_disabled.rs"]
mod probe;

pub use effort::diagnostics_with_files_and_sources;

pub const RULE_ID: &str = "declared-payload-compatibility";

/// Options for a configured declaration compatibility rule.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeclaredPayloadCompatibilityOptions {
    #[serde(default)]
    pub contracts: Vec<DeclaredPayloadContract>,
}

/// Each producer schema must be a subset of its consumer schema.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeclaredPayloadContract {
    pub name: String,
    pub producer: DeclaredPayloadSchema,
    pub consumer: DeclaredPayloadSchema,
}

/// A root-relative visible JSON document and optional RFC 6901 pointer.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeclaredPayloadSchema {
    pub file: String,
    #[serde(default)]
    pub pointer: String,
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    check_with_files_and_sources(root, config, files, &super::source_store_for_files(files))
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
) -> Result<Vec<RuleFinding>> {
    // The filesystem compatibility boundary applies suppression in its caller.
    Ok(diagnostics_with_files_and_sources(root, config, files, sources, true)?.findings)
}

fn check_contract(
    root: &Path,
    contract: &DeclaredPayloadContract,
    allowed: &BTreeSet<String>,
    sources: &SourceStore,
    documents: &document::Documents,
) -> std::result::Result<(), (String, String)> {
    if contract.name.trim().is_empty() {
        return Err((
            contract.producer.file.clone(),
            "unproven: name must be nonempty".into(),
        ));
    }
    let producer = references::load(root, &contract.producer, allowed, sources, documents)
        .map_err(|error| {
            (
                contract.producer.file.clone(),
                format!("unproven producer: {error}"),
            )
        })?;
    let consumer = references::load(root, &contract.consumer, allowed, sources, documents)
        .map_err(|error| {
            (
                contract.consumer.file.clone(),
                format!("unproven consumer: {error}"),
            )
        })?;
    compare::subset(&producer, &consumer, "$").map_err(|error| {
        (
            contract.producer.file.clone(),
            format!(
                "incompatible with {}{}: {error}",
                contract.consumer.file, contract.consumer.pointer
            ),
        )
    })
}

#[cfg(test)]
mod effort_tests;
#[cfg(test)]
mod tests;
