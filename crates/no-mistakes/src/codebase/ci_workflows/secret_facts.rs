//! Request-owned secret syntax facts; inventory resolution stays in the rule.

mod document;
mod inventory_locations;
mod references;
#[cfg(test)]
#[path = "secret_facts_tests.rs"]
mod tests;
mod workflow;
pub(crate) use inventory_locations::InventoryLocations;

use crate::codebase::ts_source::SourceStore;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Default)]
pub(crate) struct SecretDocuments {
    pub(crate) workflows: BTreeMap<String, DocumentFacts>,
    pub(crate) actions: BTreeMap<String, DocumentFacts>,
}

#[derive(Default)]
pub(crate) struct DocumentFacts {
    pub(crate) references: Vec<Reference>,
    pub(crate) jobs: BTreeMap<String, Scope>,
    pub(crate) incomplete: Vec<(usize, String)>,
    pub(crate) composite: bool,
}

#[derive(Default)]
pub(crate) struct Scope {
    pub(crate) references: Vec<Reference>,
    pub(crate) environment: Environment,
    pub(crate) line: usize,
}

#[derive(Default)]
pub(crate) enum Environment {
    #[default]
    Repository,
    Named(String),
    Unknown,
}

pub(crate) struct Reference {
    pub(crate) name: Option<String>,
    pub(crate) line: usize,
}

pub(crate) fn collect(
    root: &Path,
    workflows: &super::ParsedWorkflowSet,
    actions: &super::ParsedWorkflowSet,
    sources: &SourceStore,
) -> SecretDocuments {
    let gather = |documents: &super::ParsedWorkflowSet, action| {
        documents
            .documents
            .par_iter()
            .map(|entry| {
                let facts = match &entry.value {
                    Ok(value) => match sources.read_path(&root.join(&entry.path)) {
                        Ok(source) => workflow::collect(value, &source, action),
                        Err(error) => failed(error.to_string()),
                    },
                    Err(error) => failed(error.message.clone()),
                };
                (entry.path.clone(), facts)
            })
            .collect()
    };
    SecretDocuments {
        workflows: gather(workflows, false),
        actions: gather(actions, true),
    }
}

fn failed(message: String) -> DocumentFacts {
    DocumentFacts {
        incomplete: vec![(1, format!("document unavailable: {message}"))],
        ..Default::default()
    }
}
