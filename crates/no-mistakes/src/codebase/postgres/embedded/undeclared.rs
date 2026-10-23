//! Diagnostic identities never confer executor proof or trigger SQL evaluation.
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UndeclaredExecutorType {
    pub type_name: String,
    pub module: String,
    pub line: u32,
    pub span: (u32, u32),
    pub(crate) owner: Option<u32>,
    pub(crate) explicit_annotation: bool,
    pub(crate) configured_type: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct UndeclaredExecutorTypeSummary {
    #[serde(rename = "type")]
    pub type_name: String,
    pub module: String,
    pub count: usize,
}

pub fn summarize_undeclared_executor_types<'a>(
    occurrences: impl IntoIterator<Item = &'a UndeclaredExecutorType>,
) -> Vec<UndeclaredExecutorTypeSummary> {
    summarize_receivers(occurrences, false)
}

pub fn summarize_unresolved_executor_receivers<'a>(
    occurrences: impl IntoIterator<Item = &'a UndeclaredExecutorType>,
) -> Vec<UndeclaredExecutorTypeSummary> {
    summarize_receivers(occurrences, true)
}

fn summarize_receivers<'a>(
    occurrences: impl IntoIterator<Item = &'a UndeclaredExecutorType>,
    configured: bool,
) -> Vec<UndeclaredExecutorTypeSummary> {
    let mut counts = std::collections::BTreeMap::new();
    for occurrence in occurrences {
        // Relative syntax alone does not prove the configured module's ownership.
        if occurrence.owner.is_some() || occurrence.configured_type != configured {
            continue;
        }
        *counts
            .entry((occurrence.type_name.clone(), occurrence.module.clone()))
            .or_insert(0) += 1;
    }
    counts
        .into_iter()
        .map(
            |((type_name, module), count)| UndeclaredExecutorTypeSummary {
                type_name,
                module,
                count,
            },
        )
        .collect()
}
