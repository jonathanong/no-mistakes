use crate::codebase::ts_resolver::ImportResolverFacade;
use crate::codebase::ts_source::FrozenPathRemapper;
use crate::config::v2::schema::{JobStateAsRecordConfig, QueuesTopLevelConfig};
use crate::queue::extract_model::{FileFacts, ProducerBinding};
use crate::queue::graph_model::{CheckFinding, CheckSeverity};
use crate::queue::graph_resolution::{local_queues, resolved_binding};
use crate::queue::source::relative_string;
use crate::queue::test_scope::ConfiguredTestScope;
use crate::queue::types::{Diagnostic, DiagnosticKind, Severity};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[path = "job_state_actions.rs"]
mod actions;
#[path = "job_state_expr.rs"]
mod expr;
#[path = "job_state_extract.rs"]
mod extract;
#[path = "job_state_listing.rs"]
mod listing;
#[path = "job_state_stmts.rs"]
mod stmts;

pub(crate) use extract::collect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum JobStatePattern {
    GetJobStateGuard,
    ListingFeedsAction,
}

#[derive(Debug, Clone)]
pub(crate) struct JobStateSite {
    pub line: usize,
    pub binding: ProducerBinding,
    /// When the guarded/fed action is `.add` / `.addBulk`, its receiver must also
    /// resolve to a configured queue (so `Set.add` does not trigger a finding).
    pub action_binding: Option<ProducerBinding>,
    pub queue_file: Option<PathBuf>,
    pub queue_name: Option<String>,
    pub unresolved_receiver: bool,
    pub pattern: JobStatePattern,
}

const UNRESOLVED_RECEIVER_MESSAGE: &str =
    "job-state-as-record was not judged because the queue receiver did not resolve";

pub(crate) fn stamp(facts: &mut HashMap<PathBuf, FileFacts>, queues: &QueuesTopLevelConfig) {
    for file in facts.values_mut() {
        file.job_state_as_record
            .clone_from(&queues.job_state_as_record);
    }
}

pub(in crate::queue) fn resolve_sites<R: ImportResolverFacade>(
    facts: &mut HashMap<PathBuf, FileFacts>,
    queue_defs: &HashMap<PathBuf, HashMap<String, String>>,
    definitions: &HashMap<String, PathBuf>,
    resolver: &R,
    remapper: &FrozenPathRemapper,
) {
    if !facts
        .values()
        .any(|file| file.job_state_as_record.is_some() && !file.job_state_sites.is_empty())
    {
        return;
    }
    let mut resolved = Vec::new();
    let mut unresolved = Vec::new();
    for (path, file) in facts.iter() {
        if file.job_state_as_record.is_none() || file.job_state_sites.is_empty() {
            continue;
        }
        let local = local_queues(path, file, queue_defs, definitions, resolver, remapper);
        for (index, site) in file.job_state_sites.iter().enumerate() {
            let Some(queue) = resolved_binding(
                &local,
                facts,
                resolver,
                remapper,
                path,
                &site.binding,
                definitions,
            ) else {
                unresolved.push((path.clone(), index));
                continue;
            };
            if let Some(action) = &site.action_binding {
                if resolved_binding(&local, facts, resolver, remapper, path, action, definitions)
                    .is_none()
                {
                    continue;
                }
            }
            resolved.push((path.clone(), index, queue));
        }
    }
    for (path, index, queue) in resolved {
        let site = &mut facts
            .get_mut(&path)
            .expect("resolved path came from facts")
            .job_state_sites[index];
        site.queue_file = Some(queue.queue_file);
        site.queue_name = Some(queue.queue_name);
    }
    for (path, index) in unresolved {
        let site = &mut facts
            .get_mut(&path)
            .expect("unresolved path came from facts")
            .job_state_sites[index];
        site.unresolved_receiver = true;
    }
}

pub(super) fn findings_and_diagnostics(
    root: &Path,
    facts: &HashMap<PathBuf, FileFacts>,
    test_scope: &ConfiguredTestScope,
) -> (Vec<CheckFinding>, Vec<Diagnostic>) {
    let mut out = Vec::new();
    let mut diags = Vec::new();
    for (path, file) in facts {
        if test_scope.is_match(root, path) {
            continue;
        }
        let Some(policy) = &file.job_state_as_record else {
            continue;
        };
        if file.job_state_sites.is_empty() || is_allowed(root, path, policy) {
            continue;
        }
        for site in &file.job_state_sites {
            let Some(queue_name) = &site.queue_name else {
                if site.unresolved_receiver {
                    diags.push(Diagnostic {
                        severity: Severity::Warning,
                        file: relative_string(root, path),
                        line: site.line,
                        message: UNRESOLVED_RECEIVER_MESSAGE.to_string(),
                        kind: DiagnosticKind::AnalysisNote,
                    });
                }
                continue;
            };
            out.push(CheckFinding {
                kind: "job-state-as-record".to_string(),
                file: relative_string(root, path),
                line: site.line,
                queue_file: site
                    .queue_file
                    .as_ref()
                    .map(|queue| relative_string(root, queue)),
                queue_name: Some(queue_name.clone()),
                job: None,
                message: message(site.pattern).to_string(),
                severity: Some(CheckSeverity::Advisory),
            });
        }
    }
    (out, diags)
}

fn is_allowed(root: &Path, path: &Path, policy: &JobStateAsRecordConfig) -> bool {
    policy
        .allow_globset()
        .is_some_and(|globset| globset.is_match(relative_string(root, path)))
}

fn message(pattern: JobStatePattern) -> &'static str {
    match pattern {
        JobStatePattern::GetJobStateGuard => {
            "queue job state from getJob() guards retry/remove/add; read durable work state from the database instead"
        }
        JobStatePattern::ListingFeedsAction => {
            "failed or completed queue listing feeds add/addBulk/retry; select pending work from the database instead"
        }
    }
}
