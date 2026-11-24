use super::super::extract_model::{
    FileFacts, JobOptionEvidence, JobOptionFinding, JobOptionSite, LocalQueue, QueueDefaultOptions,
    RequiredJobOptions,
};
use super::retention::{self, RetentionValue};
use crate::fx::FxHashMap;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const CALL_OPEN: &str =
    "job options were not judged because the options object is not statically known";
const DEFAULTS_OPEN: &str =
    "job options were not judged because defaultJobOptions is not statically known";
const NAME_OPEN: &str = "job options were not judged because the queue name did not resolve";

pub(crate) fn judge_into_facts(facts: &mut HashMap<PathBuf, FileFacts>) {
    if !facts
        .values()
        .any(|file| file.required_job_options.is_some() || file.unbounded_job_retention.is_some())
    {
        return;
    }
    let defaults = index_defaults(facts);
    let mut notes = Vec::new();
    for (path, file) in facts.iter() {
        let required = file.required_job_options.as_ref();
        let retention = file.unbounded_job_retention.as_deref().unwrap_or_default();
        for site in &file.job_option_sites {
            if site.binding.is_some() && site.queue_name.is_none() {
                continue;
            }
            let inherited = site
                .queue_file
                .as_deref()
                .and_then(|file| defaults.get(file))
                .and_then(|queues| queues.get(site.queue_name.as_deref()?));
            let unbounded = retention
                .iter()
                .filter(|key| retention::unbounded_at_site(site, inherited, key))
                .map(String::as_str)
                .collect::<Vec<_>>();
            if !unbounded.is_empty() {
                let message = retention::message(&unbounded);
                notes.push(site_finding(path, site, retention::KIND, message));
            }
            if let Some(note) = required
                .and_then(|policy| judge_required(path, site, policy, inherited, &unbounded))
            {
                notes.push(note);
            }
        }
        if !retention.is_empty() {
            for options in &file.queue_defaults {
                let unbounded = retention
                    .iter()
                    .filter(|key| options.evidence.retention.get(key) == RetentionValue::Unbounded)
                    .map(String::as_str)
                    .collect::<Vec<_>>();
                if !unbounded.is_empty() {
                    notes.push(Note {
                        path: path.clone(),
                        line: options.line,
                        body: NoteBody::Finding(JobOptionFinding {
                            kind: retention::KIND,
                            line: options.line,
                            queue_file: Some(path.clone()),
                            queue_name: default_queue_name(file, options).map(str::to_owned),
                            job: None,
                            message: retention::message(&unbounded),
                        }),
                    });
                }
            }
        }
    }
    for note in notes {
        let file = facts.get_mut(&note.path).unwrap();
        match note.body {
            NoteBody::Diagnostic(message) => file.diagnostics.push((note.line, message)),
            NoteBody::Finding(finding) => file.job_option_findings.push(finding),
        }
    }
}

fn default_queue_name<'a>(
    file: &'a FileFacts,
    options: &'a QueueDefaultOptions,
) -> Option<&'a str> {
    options.queue_name.as_deref().or_else(|| {
        match file.queue_declarations.get(&options.declaration)? {
            LocalQueue::Known(name) => Some(name.as_str()),
            _ => None,
        }
    })
}

fn index_defaults(
    facts: &HashMap<PathBuf, FileFacts>,
) -> FxHashMap<&Path, FxHashMap<&str, JobOptionEvidence>> {
    let mut out = FxHashMap::<_, FxHashMap<_, JobOptionEvidence>>::default();
    for (path, file) in facts {
        for options in &file.queue_defaults {
            let Some(name) = default_queue_name(file, options) else {
                continue;
            };
            out.entry(path.as_path())
                .or_default()
                .entry(name)
                .and_modify(|entry| {
                    entry.open |= options.evidence.open;
                    entry.keys.extend(options.evidence.keys.iter().cloned());
                    entry.retention.reconcile(options.evidence.retention);
                })
                .or_insert_with(|| options.evidence.clone());
        }
    }
    out
}

struct Note {
    path: PathBuf,
    line: usize,
    body: NoteBody,
}

enum NoteBody {
    Diagnostic(String),
    Finding(JobOptionFinding),
}

fn judge_required(
    path: &Path,
    site: &JobOptionSite,
    policy: &RequiredJobOptions,
    defaults: Option<&JobOptionEvidence>,
    unbounded: &[&str],
) -> Option<Note> {
    if site.queue_name.is_none() && !policy.by_queue.is_empty() {
        return Some(diagnostic(path, site.line, NAME_OPEN));
    }
    let required = policy.for_queue(site.queue_name.as_deref());
    if required.is_empty() {
        return None;
    }
    if site.evidence.open {
        return Some(diagnostic(path, site.line, CALL_OPEN));
    }
    let mut missing = required
        .iter()
        .map(String::as_str)
        .filter(|key| !site.evidence.keys.contains(*key) && !unbounded.contains(key))
        .collect::<Vec<_>>();
    if missing.is_empty() {
        return None;
    }
    if let Some(defaults) = defaults {
        if defaults.open {
            return Some(diagnostic(path, site.line, DEFAULTS_OPEN));
        }
        missing.retain(|key| !defaults.keys.contains(*key));
    }
    (!missing.is_empty()).then(|| {
        site_finding(
            path,
            site,
            "missing-job-option",
            format!("job options missing {}", missing.join(", ")),
        )
    })
}

fn diagnostic(path: &Path, line: usize, message: &str) -> Note {
    Note {
        path: path.to_path_buf(),
        line,
        body: NoteBody::Diagnostic(message.to_string()),
    }
}

fn site_finding(path: &Path, site: &JobOptionSite, kind: &'static str, message: String) -> Note {
    Note {
        path: path.to_path_buf(),
        line: site.line,
        body: NoteBody::Finding(JobOptionFinding {
            kind,
            line: site.line,
            queue_file: site.queue_file.clone(),
            queue_name: site.queue_name.clone(),
            job: site.job.clone(),
            message,
        }),
    }
}
