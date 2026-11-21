use super::super::extract_model::{
    FileFacts, JobOptionEvidence, JobOptionFinding, JobOptionSite, RequiredJobOptions,
};
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

const CALL_OPEN: &str =
    "job options were not judged because the options object is not statically known";
const DEFAULTS_OPEN: &str =
    "job options were not judged because defaultJobOptions is not statically known";
const NAME_OPEN: &str = "job options were not judged because the queue name did not resolve";

pub(crate) fn judge_into_facts(facts: &mut HashMap<PathBuf, FileFacts>) {
    if !facts
        .values()
        .any(|file| file.required_job_options.is_some())
    {
        return;
    }
    let defaults = index_defaults(facts);
    let mut notes = Vec::new();
    for (path, file) in facts.iter() {
        let Some(policy) = file.required_job_options.clone() else {
            continue;
        };
        for site in &file.job_option_sites {
            if let Some(note) = judge_site(path, site, &policy, &defaults) {
                notes.push(note);
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

fn index_defaults(
    facts: &HashMap<PathBuf, FileFacts>,
) -> HashMap<(PathBuf, String), JobOptionEvidence> {
    let mut out: HashMap<(PathBuf, String), JobOptionEvidence> = HashMap::new();
    for (path, file) in facts {
        for defaults in &file.queue_defaults {
            let name = defaults.queue_name.clone().or_else(|| {
                defaults
                    .binding
                    .as_ref()
                    .and_then(|binding| file.queue_bindings.get(binding).cloned())
            });
            let Some(name) = name else {
                continue;
            };
            let entry = out.entry((path.clone(), name)).or_default();
            entry.open |= defaults.evidence.open;
            entry.keys.extend(defaults.evidence.keys.iter().cloned());
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

fn judge_site(
    path: &Path,
    site: &JobOptionSite,
    policy: &RequiredJobOptions,
    defaults: &HashMap<(PathBuf, String), JobOptionEvidence>,
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
    if missing(required, &site.evidence.keys).is_empty() {
        return None;
    }
    let defaults = site.queue_file.as_ref().and_then(|file| {
        site.queue_name
            .as_ref()
            .map(|name| defaults.get(&(file.clone(), name.clone())))
    });
    let Some(Some(defaults)) = defaults else {
        return Some(finding(path, site, required, &site.evidence.keys));
    };
    if defaults.open {
        return Some(diagnostic(path, site.line, DEFAULTS_OPEN));
    }
    let mut present = site.evidence.keys.clone();
    present.extend(defaults.keys.iter().cloned());
    if missing(required, &present).is_empty() {
        return None;
    }
    Some(finding(path, site, required, &present))
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
