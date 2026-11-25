use crate::edge_index::PreparedRelationshipIndex;
use crate::queue::extract::FileFacts;
use crate::queue::source::relative_string;
use crate::queue::types::{
    Diagnostic, DiagnosticKind, Edge, EdgeKind, JobKey, QueueJobNode, QueueKey, QueueProducer,
    QueueWorker, RelationshipNode, Severity,
};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckSeverity {
    Error,
    Advisory,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckFinding {
    pub kind: String,
    pub file: String,
    pub line: usize,
    pub queue_file: Option<String>,
    pub queue_name: Option<String>,
    pub job: Option<String>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<CheckSeverity>,
}

impl CheckFinding {
    pub fn is_advisory(&self) -> bool {
        self.severity == Some(CheckSeverity::Advisory)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckReport<'a> {
    pub findings: &'a [CheckFinding],
    pub diagnostics: &'a [Diagnostic],
}

pub fn format_diagnostic(diagnostic: &Diagnostic) -> String {
    format!(
        "{}: {}:{} {}",
        diagnostic.severity, diagnostic.file, diagnostic.line, diagnostic.message
    )
}

pub fn is_check_warning(diagnostic: &Diagnostic) -> bool {
    matches!(
        diagnostic.kind,
        DiagnosticKind::WorkerRateLimitUnresolved | DiagnosticKind::CheckAbstention
    )
}

pub fn format_aggregate_warning(diagnostic: &Diagnostic) -> String {
    format!(
        "queues-check: {}:{} {}",
        diagnostic.file, diagnostic.line, diagnostic.message
    )
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectReport {
    pub producers: Vec<QueueProducer>,
    pub workers: Vec<QueueWorker>,
    pub jobs: Vec<QueueJobNode>,
    pub edges: Vec<Edge>,
    pub diagnostics: Vec<Diagnostic>,
    pub check: Vec<CheckFinding>,
}

/// A queue report coupled to its request-scoped typed relationship index.
///
/// This is hidden from the stable serialized API. CLI and N-API projections
/// use it so traversal never recovers node types from `queueFile#job` strings.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct PreparedProjectReport {
    pub(crate) report: ProjectReport,
    pub(crate) relationships: PreparedRelationshipIndex<RelationshipNode, EdgeKind>,
}

impl PreparedProjectReport {
    pub fn report(&self) -> &ProjectReport {
        &self.report
    }
}

#[derive(Debug, Clone)]
pub(super) struct InternalProducer {
    pub site: crate::queue::extract::ProducerSite,
    pub queue: Option<QueueKey>,
}

#[derive(Debug, Clone)]
pub(super) struct InternalWorker {
    pub site: crate::queue::extract::WorkerSite,
    pub queue: Option<QueueKey>,
    pub reachable_files: HashSet<PathBuf>,
}

pub(super) fn diagnostics(
    root: &Path,
    facts: &HashMap<PathBuf, FileFacts>,
    producers: &[InternalProducer],
    workers: &[InternalWorker],
    test_scope: &crate::queue::test_scope::ConfiguredTestScope,
) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for (path, facts) in facts {
        if !test_scope.is_match(root, path) {
            out.extend(
                facts
                    .check_abstentions
                    .iter()
                    .map(|&(line, message)| Diagnostic {
                        kind: DiagnosticKind::CheckAbstention,
                        severity: Severity::Warning,
                        file: relative_string(root, path),
                        line,
                        message: message.to_string(),
                    }),
            );
        }
        for (line, message) in &facts.diagnostics {
            out.push(Diagnostic {
                kind: DiagnosticKind::AnalysisNote,
                severity: Severity::Warning,
                file: relative_string(root, path),
                line: *line,
                message: message.clone(),
            });
        }
    }
    out.extend(
        producers
            .iter()
            .filter_map(|p| unresolved_producer(root, p)),
    );
    out.extend(workers.iter().filter_map(|w| unresolved_worker(root, w)));
    dedup_sorted(out)
}

pub(super) fn node_name(root: &Path, job: &JobKey) -> String {
    format!("{}#{}", relative_string(root, &job.queue_file), job.job)
}

pub(super) fn build_filter(filters: &[String]) -> anyhow::Result<Option<GlobSet>> {
    if filters.is_empty() {
        return Ok(None);
    }
    let mut builder = GlobSetBuilder::new();
    for filter in filters {
        builder.add(GlobBuilder::new(filter).literal_separator(false).build()?);
    }
    Ok(Some(builder.build()?))
}

pub(super) fn dedup_sorted<T: Ord>(mut values: Vec<T>) -> Vec<T> {
    values.sort();
    values.dedup();
    values
}

fn unresolved_producer(root: &Path, producer: &InternalProducer) -> Option<Diagnostic> {
    (producer.queue.is_none() || producer.site.job.is_none()).then(|| Diagnostic {
        kind: DiagnosticKind::AnalysisNote,
        severity: Severity::Warning,
        file: relative_string(root, &producer.site.file),
        line: producer.site.line,
        message: "dynamic or unresolved queue producer".to_string(),
    })
}

fn unresolved_worker(root: &Path, worker: &InternalWorker) -> Option<Diagnostic> {
    worker.queue.is_none().then(|| Diagnostic {
        kind: DiagnosticKind::AnalysisNote,
        severity: Severity::Warning,
        file: relative_string(root, &worker.site.file),
        line: worker.site.line,
        message: "dynamic or unresolved queue worker".to_string(),
    })
}
