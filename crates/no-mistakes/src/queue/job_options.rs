use super::extract_model::{FileFacts, JobOptionEvidence, RequiredJobOptions};
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
#[path = "job_options_site.rs"]
mod site;
#[path = "job_options_walk.rs"]
mod walk;

pub(crate) use defaults::{
    note_argument_options, note_constructed_defaults, note_factory_defaults,
};
pub(crate) use judge::judge_into_facts;
pub(crate) use note::{note_call, note_consts};

pub(crate) fn stamp(facts: &mut HashMap<PathBuf, FileFacts>, queues: &QueuesTopLevelConfig) {
    if queues.required_job_options.is_empty() && queues.required_job_options_by_queue.is_empty() {
        return;
    }
    let policy = RequiredJobOptions {
        global: queues.required_job_options.clone(),
        by_queue: queues.required_job_options_by_queue.clone(),
    };
    for file in facts.values_mut() {
        file.required_job_options = Some(policy.clone());
    }
}

pub(crate) fn stamp_unconfigured(root: &Path, facts: &mut HashMap<PathBuf, FileFacts>) {
    if facts
        .values()
        .any(|file| file.required_job_options.is_some())
    {
        return;
    }
    if let Ok(config) = crate::config::v2::load_v2_config(root, None) {
        stamp(facts, &config.queues);
    }
}

pub(super) fn finish(facts: &mut FileFacts) {
    let objects = walk::resolve_objects(&std::mem::take(&mut facts.const_objects));
    let mut opts = std::mem::take(&mut facts.const_opts);
    apply_map(&mut opts, &objects);
    let mut defaults = std::mem::take(&mut facts.const_queue_defaults);
    apply_map(&mut defaults, &objects);
    for recorded in &mut facts.queue_defaults {
        if let Some(name) = recorded.container.take() {
            recorded.evidence = if let Some(found) = defaults.get(&name) {
                found.clone()
            } else if objects.contains_key(&name) {
                JobOptionEvidence::default()
            } else {
                JobOptionEvidence::open()
            };
        } else {
            walk::apply_evidence(&mut recorded.evidence, &objects);
        }
    }
    for site in &mut facts.job_option_sites {
        if site.evidence.member.as_deref() == Some("opts") {
            let name = site.evidence.spreads[0].clone();
            if let Some(found) = opts.get(&name) {
                site.evidence = found.clone();
            } else if objects.contains_key(&name) {
                site.evidence.member = None;
                site.evidence.spreads.clear();
            } else {
                site.evidence = JobOptionEvidence::open();
            }
        } else {
            walk::apply_evidence(&mut site.evidence, &objects);
        }
    }
}

fn apply_map(
    map: &mut HashMap<String, JobOptionEvidence>,
    objects: &HashMap<String, JobOptionEvidence>,
) {
    let mut names: Vec<String> = map.keys().cloned().collect();
    names.sort();
    for name in names {
        if let Some(mut evidence) = map.remove(&name) {
            walk::apply_evidence(&mut evidence, objects);
            map.insert(name, evidence);
        }
    }
}

pub(crate) fn defined_queue_file(
    facts: &HashMap<PathBuf, FileFacts>,
    queue_name: &str,
) -> Option<PathBuf> {
    facts
        .iter()
        .filter(|(_, file)| {
            crate::queue::factory_resolve::defined_queue_names(file).any(|name| name == queue_name)
        })
        .map(|(path, _)| path)
        .min()
        .cloned()
}

pub(super) fn local_checks(root: &Path, facts: &HashMap<PathBuf, FileFacts>) -> Vec<CheckFinding> {
    let mut out = Vec::new();
    for (path, file) in facts {
        for check in &file.checks {
            out.push(CheckFinding {
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
                kind: "missing-job-option".to_string(),
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
