use super::extract_model::{FileFacts, RequiredJobOptions};
use super::graph_model::CheckFinding;
use super::source::relative_string;
use super::test_scope::ConfiguredTestScope;
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
#[path = "job_options_retention_read.rs"]
mod retention_read;
#[path = "job_options_site.rs"]
mod site;
#[path = "job_options_traversal.rs"]
mod traversal;
#[path = "job_options_walk.rs"]
mod walk;

pub(crate) use defaults::{
    note_argument_options, note_constructed_defaults, note_factory_defaults,
};
pub(crate) use judge::judge_into_facts;
pub(crate) use note::note_call;
pub(super) use walk::{finish, read as read_evidence};

pub(crate) fn stamp(facts: &mut HashMap<PathBuf, FileFacts>, queues: &QueuesTopLevelConfig) {
    let policy = (!queues.required_job_options.is_empty()
        || !queues.required_job_options_by_queue.is_empty())
    .then(|| RequiredJobOptions {
        global: queues.required_job_options.clone(),
        by_queue: queues.required_job_options_by_queue.clone(),
    });
    let retention = queues
        .unbounded_job_retention
        .clone()
        .filter(|config| config.is_enabled());
    for file in facts.values_mut() {
        file.required_job_options.clone_from(&policy);
        file.unbounded_job_retention.clone_from(&retention);
    }
    super::job_state::stamp(facts, queues);
}

pub(super) fn local_checks(
    root: &Path,
    facts: &HashMap<PathBuf, FileFacts>,
    test_scope: &ConfiguredTestScope,
) -> Vec<CheckFinding> {
    let mut out = Vec::new();
    for (path, file) in facts {
        if test_scope.is_match(root, path) {
            continue;
        }
        for check in &file.checks {
            out.push(CheckFinding {
                kind: check.kind.clone(),
                file: relative_string(root, path),
                line: check.line,
                queue_file: None,
                queue_name: None,
                job: check.job.clone(),
                message: check.message.clone(),
                severity: None,
            });
        }
        for finding in &file.job_option_findings {
            out.push(CheckFinding {
                kind: finding.kind.clone(),
                file: relative_string(root, path),
                line: finding.line,
                queue_file: finding
                    .queue_file
                    .as_ref()
                    .map(|queue| relative_string(root, queue)),
                queue_name: finding.queue_name.clone(),
                job: finding.job.clone(),
                message: finding.message.clone(),
                severity: None,
            });
        }
    }
    out
}
