use super::super::extract_model::{FileFacts, JobOptionFinding, JobOptionSite, RequiredJobOptions};
use super::super::test_scope::ConfiguredTestScope;
use super::defaults::{index_defaults, DefaultsIndex};
use super::retention;
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

const CALL_OPEN: &str =
    "job options were not judged because the options object is not statically known";
const DEFAULTS_OPEN: &str =
    "job options were not judged because defaultJobOptions is not statically known";
const DEFAULTS_AMBIGUOUS: &str =
    "job options were not judged because defaultJobOptions is ambiguous across multiple queue definitions";
const NAME_OPEN: &str = "job options were not judged because the queue name did not resolve";

pub(crate) fn judge_into_facts(
    root: &Path,
    facts: &mut HashMap<PathBuf, FileFacts>,
    test_scope: &ConfiguredTestScope,
) {
    let any_required = facts
        .values()
        .any(|file| file.required_job_options.is_some());
    // Stamping applies the selected retention policy to every file for this root.
    let retention = facts
        .values()
        .find_map(|file| file.unbounded_job_retention.as_ref());
    if !any_required && retention.is_none() {
        return;
    }
    let allow = retention.and_then(|policy| policy.allow_globset());
    let defaults = index_defaults(facts);
    let mut notes = Vec::new();
    for (path, file) in facts.iter() {
        if test_scope.is_match(root, path) {
            continue;
        }
        let retention_policy = file.unbounded_job_retention.as_ref();
        if let Some(retention_policy) = retention_policy {
            for note in
                retention::judge_file_defaults(root, path, file, retention_policy, allow.as_ref())
            {
                notes.push(Note {
                    path: note.path,
                    line: note.finding.line,
                    body: NoteBody::Finding(note.finding),
                });
            }
        }
        let covered = retention::covered_keys(root, path, retention_policy, allow.as_ref());
        for site in &file.job_option_sites {
            if let Some(retention_policy) = retention_policy {
                let judgment = retention::judge_site(
                    root,
                    path,
                    site,
                    retention_policy,
                    allow.as_ref(),
                    &defaults,
                );
                if let Some(note) = judgment.finding {
                    notes.push(Note {
                        path: note.path,
                        line: note.finding.line,
                        body: NoteBody::Finding(note.finding),
                    });
                }
                for message in judgment.abstentions.into_iter().flatten() {
                    notes.push(Note {
                        path: path.clone(),
                        line: site.line,
                        body: NoteBody::Abstention(message),
                    });
                }
            }
            if let Some(policy) = &file.required_job_options {
                if let Some(note) = judge_site(path, site, policy, &covered, &defaults) {
                    notes.push(note);
                }
            }
        }
    }
    for note in notes {
        let file = facts.get_mut(&note.path).unwrap();
        match note.body {
            NoteBody::Abstention(message) => file.check_abstentions.push((note.line, message)),
            NoteBody::Diagnostic(message) => file.diagnostics.push((note.line, message)),
            NoteBody::Finding(finding) => file.job_option_findings.push(finding),
        }
    }
}

struct Note {
    path: PathBuf,
    line: usize,
    body: NoteBody,
}

enum NoteBody {
    Abstention(&'static str),
    Diagnostic(String),
    Finding(JobOptionFinding),
}

fn judge_site(
    path: &Path,
    site: &JobOptionSite,
    policy: &RequiredJobOptions,
    covered_by_retention: &[&str],
    defaults: &DefaultsIndex,
) -> Option<Note> {
    if site.binding.is_some() && site.queue_name.is_none() {
        return None;
    }
    if site.queue_name.is_none() && !policy.by_queue.is_empty() {
        return Some(diagnostic(path, site.line, NAME_OPEN));
    }
    let required: Vec<String> = policy
        .for_queue(site.queue_name.as_deref())
        .iter()
        .filter(|key| !covered_by_retention.contains(&key.as_str()))
        .cloned()
        .collect();
    if required.is_empty() {
        return None;
    }
    if site.evidence.open {
        return Some(diagnostic(path, site.line, CALL_OPEN));
    }
    if missing(&required, &site.evidence.keys).is_empty() {
        return None;
    }
    if site.ambiguous_defaults {
        return Some(diagnostic(path, site.line, DEFAULTS_AMBIGUOUS));
    }
    let Some(defaults) = defaults.lookup_evidence(site) else {
        return Some(finding(path, site, &required, &site.evidence.keys));
    };
    let Ok(defaults) = defaults else {
        return Some(diagnostic(path, site.line, DEFAULTS_AMBIGUOUS));
    };
    if defaults.open {
        return Some(diagnostic(path, site.line, DEFAULTS_OPEN));
    }
    let mut present = site.evidence.keys.clone();
    present.extend(defaults.keys.iter().cloned());
    if missing(&required, &present).is_empty() {
        return None;
    }
    Some(finding(path, site, &required, &present))
}

fn missing<'a>(required: &'a [String], present: &BTreeSet<String>) -> Vec<&'a str> {
    required
        .iter()
        .filter(|key| !present.contains(key.as_str()))
        .map(String::as_str)
        .collect()
}

fn diagnostic(path: &Path, line: usize, message: &str) -> Note {
    Note {
        path: path.to_path_buf(),
        line,
        body: NoteBody::Diagnostic(message.to_string()),
    }
}

fn finding(
    path: &Path,
    site: &JobOptionSite,
    required: &[String],
    present: &BTreeSet<String>,
) -> Note {
    Note {
        path: path.to_path_buf(),
        line: site.line,
        body: NoteBody::Finding(JobOptionFinding {
            kind: "missing-job-option".to_string(),
            line: site.line,
            queue_file: site.queue_file.clone(),
            queue_name: site.queue_name.clone(),
            job: site.job.clone(),
            message: format!(
                "job options missing {}",
                missing(required, present).join(", ")
            ),
        }),
    }
}
