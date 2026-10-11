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
mod numeric;
mod references;
mod schema;

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
    let documents = document::Documents::default();
    let results: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| {
            let options: DeclaredPayloadCompatibilityOptions = rule.try_rule_options()?;
            let allowed = super::path_filter::filter_rule_files(root, config, rule, files)?;
            let allowed: BTreeSet<_> = allowed
                .iter()
                .map(|p| relative_slash_path(root, p))
                .collect();
            Ok(options
                .contracts
                .par_iter()
                .filter_map(|contract| {
                    let result = check_contract(root, contract, &allowed, sources, &documents);
                    result.err().map(|(file, reason)| RuleFinding {
                        rule: RULE_ID.to_string(),
                        line: references::finding_line(root, &file, &allowed, sources),
                        file,
                        message: match &rule.message {
                            Some(message) => {
                                format!("{message}: contract `{}`: {reason}", contract.name)
                            }
                            None => format!("contract `{}`: {reason}", contract.name),
                        },
                        import: None,
                        target: Some(contract.name.clone()),
                    })
                })
                .collect())
        })
        .collect();
    let mut findings = results?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
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
mod tests;
