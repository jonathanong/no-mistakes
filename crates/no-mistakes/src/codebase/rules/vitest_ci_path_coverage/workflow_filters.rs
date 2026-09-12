mod extract;
mod step;
mod values;
mod workflow_paths;

use super::{
    globs::{selected_by, PredicateQuantifier},
    RuleFinding, RULE_ID,
};
use crate::codebase::ci_graph::discover_workflow_files_from_snapshot;
use crate::codebase::ci_workflows::{ParsedWorkflowSet, WorkflowDocumentErrorKind};
use crate::config::v2::schema::NoMistakesConfig;
use serde::Deserialize;
use std::path::Path;
use workflow_paths::{workflow_path_filters, WorkflowPathFilters};

#[cfg(test)]
mod tests;

#[derive(Clone, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct WorkflowSelector {
    pub(crate) path: String,
    pub(crate) job: String,
    pub(crate) step_id: String,
}

#[derive(Debug)]
pub(super) struct CiFilter {
    pub(super) workflow: String,
    pub(super) name: String,
    pub(super) compiled: Vec<Vec<super::globs::CompiledGlob>>,
    pub(super) quantifier: PredicateQuantifier,
    workflow_paths: WorkflowPathFilters,
}

impl CiFilter {
    pub(super) fn workflow_allows(&self, path: &str) -> bool {
        self.workflow_paths.allows(path)
    }
}

pub(super) fn ci_filters_from_snapshot_with_sources(
    root: &Path,
    config: &NoMistakesConfig,
    selectors: &[WorkflowSelector],
    snapshot: &crate::codebase::ts_source::VisiblePathSnapshot,
    sources: &crate::codebase::ts_source::SourceStore,
) -> (Vec<CiFilter>, Vec<RuleFinding>) {
    ci_filters_from_paths(
        root,
        selectors,
        discover_workflow_files_from_snapshot(root, &config.ci, snapshot),
        sources,
    )
}

pub(super) fn ci_filters_from_parsed_with_sources(
    root: &Path,
    selectors: &[WorkflowSelector],
    parsed: &ParsedWorkflowSet,
    sources: &crate::codebase::ts_source::SourceStore,
) -> (Vec<CiFilter>, Vec<RuleFinding>) {
    let mut filters = Vec::new();
    let mut findings = Vec::new();
    for document in &parsed.documents {
        let rel = &document.path;
        if !selector_allows(selectors, rel) {
            continue;
        }
        let value = match &document.value {
            Ok(value) => value,
            Err(error) => {
                let action = match error.kind {
                    WorkflowDocumentErrorKind::Read => "read workflow file",
                    WorkflowDocumentErrorKind::Parse => "parse workflow YAML",
                };
                findings.push(workflow_finding(
                    rel,
                    format!("{rel}: could not {action}: {}", error.message),
                    None,
                ));
                continue;
            }
        };
        let (workflow_filters, workflow_findings) =
            extract::from_value(root, rel, value, selectors, sources);
        filters.extend(workflow_filters);
        findings.extend(workflow_findings);
    }
    sort_filters(&mut filters);
    (filters, findings)
}

fn ci_filters_from_paths(
    root: &Path,
    selectors: &[WorkflowSelector],
    workflow_files: Vec<std::path::PathBuf>,
    sources: &crate::codebase::ts_source::SourceStore,
) -> (Vec<CiFilter>, Vec<RuleFinding>) {
    let mut filters = Vec::new();
    let mut findings = Vec::new();
    for path in workflow_files {
        let rel = crate::codebase::ci_graph::relative_slash(root, &path);
        if !selector_allows(selectors, &rel) {
            continue;
        }
        let source = match sources.read_path(&path) {
            Ok(source) => source,
            Err(error) => {
                findings.push(workflow_finding(
                    &rel,
                    format!("{rel}: could not read workflow file: {error}"),
                    None,
                ));
                continue;
            }
        };
        let (workflow_filters, workflow_findings) =
            extract::from_workflow(root, &rel, &source, selectors, sources);
        filters.extend(workflow_filters);
        findings.extend(workflow_findings);
    }
    sort_filters(&mut filters);
    (filters, findings)
}

fn selector_allows(selectors: &[WorkflowSelector], rel: &str) -> bool {
    if selectors.is_empty() {
        return true;
    }
    for selector in selectors {
        if selector.path.is_empty() || selector.path == rel {
            return true;
        }
    }
    false
}

fn sort_filters(filters: &mut [CiFilter]) {
    filters.sort_by(|a, b| (&a.workflow, &a.name).cmp(&(&b.workflow, &b.name)));
}

pub(super) fn workflow_finding(file: &str, message: String, target: Option<String>) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line: 1,
        message,
        import: None,
        target,
    }
}
