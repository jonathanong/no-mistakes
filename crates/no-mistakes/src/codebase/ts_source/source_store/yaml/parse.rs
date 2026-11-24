use super::{CachedYaml, YamlDocumentsOutcome, YamlLoadError, YamlParseOutcome};
use crate::codebase::ts_source::SourceStore;
use serde::Deserialize;
use std::sync::{Arc, OnceLock};

pub(super) fn cached_from_str(store: &SourceStore, source: &str) -> CachedYaml {
    let split = split_documents(source);
    let documents = document_view(&split);
    let single = single_view(&split);
    if documents.is_err() {
        store.increment("yaml.errors", 1);
    }
    CachedYaml::from_views(documents, single)
}

pub(super) struct SplitYaml {
    pub(super) documents: Vec<Arc<serde_yaml::Value>>,
    pub(super) failure: Option<Arc<serde_yaml::Error>>,
}

fn split_documents(source: &str) -> SplitYaml {
    let mut documents = Vec::new();
    let mut failure = None;
    for document in serde_yaml::Deserializer::from_str(source) {
        match serde_yaml::Value::deserialize(document) {
            Ok(value) => documents.push(Arc::new(value)),
            Err(error) => {
                failure = Some(Arc::new(error));
                break;
            }
        }
    }
    SplitYaml { documents, failure }
}

pub(super) fn document_view(split: &SplitYaml) -> YamlDocumentsOutcome {
    if split.documents.is_empty() {
        if let Some(error) = &split.failure {
            return Err(YamlLoadError::Syntax(Arc::clone(error)));
        }
    }
    Ok(Arc::from(split.documents.as_slice()))
}

/// `from_str` keeps one clean document. No documents is an empty value.
/// A later document, including one that does not parse, is rejected.
pub(super) fn single_view(split: &SplitYaml) -> YamlParseOutcome {
    if split.documents.len() == 1 && split.failure.is_none() {
        return Ok(Arc::clone(&split.documents[0]));
    }
    if split.documents.is_empty() {
        if let Some(error) = &split.failure {
            return Err(YamlLoadError::Syntax(Arc::clone(error)));
        }
        return Ok(null_document());
    }
    Err(YamlLoadError::Syntax(extra_document_error()))
}

fn null_document() -> Arc<serde_yaml::Value> {
    static CELL: OnceLock<Arc<serde_yaml::Value>> = OnceLock::new();
    Arc::clone(CELL.get_or_init(|| Arc::new(serde_yaml::Value::Null)))
}

fn extra_document_error() -> Arc<serde_yaml::Error> {
    static CELL: OnceLock<Arc<serde_yaml::Error>> = OnceLock::new();
    Arc::clone(CELL.get_or_init(|| {
        Arc::new(
            serde_yaml::from_str::<serde_yaml::Value>("a: 1\n---\nb: 2\n")
                .expect_err("two YAML documents are rejected"),
        )
    }))
}
