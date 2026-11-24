use super::extract_model::{FileFacts, RequiredJobOptions};
use super::graph_model::CheckFinding;
use super::source::relative_string;
use crate::config::v2::schema::QueuesTopLevelConfig;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[path = "job_options_defaults.rs"]
mod defaults;
#[path = "job_options_flow.rs"]
mod flow;
#[path = "job_options_judge.rs"]
mod judge;
#[path = "job_options_note.rs"]
mod note;
#[path = "job_options_retention.rs"]
mod retention;
#[path = "job_options_site.rs"]
mod site;
#[path = "job_options_walk.rs"]
mod walk;

pub(crate) use defaults::{
    note_argument_options, note_constructed_defaults, note_factory_defaults,
};
pub(crate) use judge::judge_into_facts;
pub(crate) use note::note_call;
pub(super) use retention::RetentionEvidence;
pub(super) use retention::KIND as RETENTION_KIND;
pub(super) use walk::{finish, read as read_evidence};

pub(crate) fn stamp(
    root: &Path,
    facts: &mut HashMap<PathBuf, FileFacts>,
    queues: &QueuesTopLevelConfig,
) -> anyhow::Result<()> {
    let policy = (!queues.required_job_options.is_empty()
        || !queues.required_job_options_by_queue.is_empty())
    .then(|| RequiredJobOptions {
        global: queues.required_job_options.clone(),
        by_queue: queues.required_job_options_by_queue.clone(),
    });
    let retention = queues.unbounded_job_retention.as_ref();
    let allow_paths = retention
        .map(|policy| {
            policy
                .allow_paths
                .iter()
                .map(|path| path.trim_start_matches("./").to_owned())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let allow = super::graph_model::build_filter(&allow_paths)?;
    let retention_keys: Option<std::sync::Arc<[String]>> = retention
        .filter(|policy| !policy.keys.is_empty())
        .map(|policy| policy.keys.clone().into());
    for (path, file) in facts {
        file.required_job_options.clone_from(&policy);
        file.unbounded_job_retention = if allow
            .as_ref()
            .is_some_and(|allow| allow.is_match(path.strip_prefix(root).unwrap_or(path)))
        {
            None
        } else {
            retention_keys.clone()
        };
    }
    Ok(())
}

pub(super) fn local_checks(root: &Path, facts: &HashMap<PathBuf, FileFacts>) -> Vec<CheckFinding> {
    let mut out = Vec::new();
    for (path, file) in facts {
        for check in &file.checks {
            out.push(CheckFinding {
                severity: if check.kind == super::job_state::KIND {
                    super::graph_model::QueueFindingSeverity::Advisory
                } else {
                    super::graph_model::QueueFindingSeverity::Blocking
                },
                kind: check.kind.clone(),
                file: relative_string(root, path),
                line: check.line,
                queue_file: None,
                queue_name: None,
                job: check.job.clone(),
                message: check.message.clone(),
            });
        }
        for finding in &file.job_option_findings {
            out.push(CheckFinding {
                severity: super::graph_model::QueueFindingSeverity::Blocking,
                kind: finding.kind.to_string(),
                file: relative_string(root, path),
                line: finding.line,
                queue_file: finding
                    .queue_file
                    .as_ref()
                    .map(|queue| relative_string(root, queue)),
                queue_name: finding.queue_name.clone(),
                job: finding.job.clone(),
                message: finding.message.clone(),
            });
        }
    }
    out
}
