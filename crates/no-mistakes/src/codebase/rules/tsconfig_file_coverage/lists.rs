use super::{finding, CompiledAuxiliary, CompiledOptions, RuleFinding};
use crate::codebase::ts_source::relative_slash_path;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const PROGRAM_KEYS: &[&str] = &["files", "include", "exclude", "references"];

pub(super) fn list_findings(
    root: &Path,
    opts: &CompiledOptions,
    tsconfigs: &BTreeSet<String>,
    candidates: &[(PathBuf, String)],
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<RuleFinding> {
    let candidate_paths = candidates
        .iter()
        .map(|(_, rel)| rel.as_str())
        .collect::<BTreeSet<_>>();
    let mut findings = Vec::new();
    for entry in &opts.allow {
        findings.extend(reasoned_path_findings(
            "allow",
            &entry.path,
            &entry.reason,
            &candidate_paths,
        ));
    }
    let tracked_files = files
        .iter()
        .map(|path| relative_slash_path(root, path))
        .collect::<BTreeSet<_>>();
    for entry in &opts.auxiliary {
        findings.extend(auxiliary_findings(
            root,
            entry,
            tsconfigs,
            &tracked_files,
            sources,
        ));
    }
    findings
}

fn reasoned_path_findings(
    kind: &str,
    path: &str,
    reason: &str,
    known: &BTreeSet<&str>,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    if reason.trim().is_empty() {
        findings.push(finding(
            path,
            format!("tsconfig-file-coverage {kind} entry `{path}` has an empty reason"),
        ));
    }
    if path.is_empty() || !known.contains(path) {
        findings.push(finding(
            path,
            format!("stale tsconfig-file-coverage {kind} entry `{path}`"),
        ));
    }
    findings
}

fn auxiliary_findings(
    root: &Path,
    entry: &CompiledAuxiliary,
    tsconfigs: &BTreeSet<String>,
    tracked_files: &BTreeSet<String>,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<RuleFinding> {
    let mut findings = reasoned_path_findings(
        "auxiliaryConfigs",
        &entry.path,
        &entry.reason,
        &tsconfigs.iter().map(String::as_str).collect(),
    );
    if !basename_matches(&entry.path, &entry.required_basename) {
        findings.push(finding(
            &entry.path,
            format!(
                "{}: auxiliary tsconfig basename must be {}",
                entry.path, entry.required_basename
            ),
        ));
    }
    if tsconfigs.contains(&entry.path) || tracked_files.contains(&entry.path) {
        findings.extend(program_key_findings(root, entry, sources));
    }
    findings
}

fn program_key_findings(
    root: &Path,
    entry: &CompiledAuxiliary,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<RuleFinding> {
    let path = root.join(&entry.path);
    let Some(source) = crate::codebase::rules::read_source(sources, &path) else {
        return vec![finding(
            &entry.path,
            format!("{}: auxiliary tsconfig is not a JSON object", entry.path),
        )];
    };
    let parsed =
        jsonc_parser::parse_to_serde_value(&source, &jsonc_parser::ParseOptions::default())
            .ok()
            .flatten();
    let Some(object) = parsed.as_ref().and_then(serde_json::Value::as_object) else {
        return vec![finding(
            &entry.path,
            format!("{}: auxiliary tsconfig is not a JSON object", entry.path),
        )];
    };
    if !PROGRAM_KEYS.iter().any(|key| object.contains_key(*key)) {
        return Vec::new();
    }
    vec![finding(
        &entry.path,
        format!(
            "{}: auxiliary tsconfig must not declare files, include, exclude, or references",
            entry.path
        ),
    )]
}

fn basename_matches(path: &str, required: &str) -> bool {
    Path::new(path).file_name().and_then(|name| name.to_str()) == Some(required)
}
