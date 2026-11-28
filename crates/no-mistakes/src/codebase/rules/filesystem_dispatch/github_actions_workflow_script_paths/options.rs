use super::RULE_ID;
use anyhow::{bail, ensure, Result};
use globset::GlobBuilder;
use serde::Deserialize;
use std::collections::BTreeSet;

pub(super) const PATH_FILTER_EVENTS: &[&str] = &["push", "pull_request", "pull_request_target"];

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) script_dirs: Vec<String>,
    pub(crate) include: Vec<String>,
    pub(crate) events: Vec<String>,
}

pub(in crate::codebase::rules) struct CompiledOptions {
    pub(super) script_dirs: Vec<String>,
    pub(super) include: Vec<String>,
    pub(super) events: Vec<String>,
}

pub(in crate::codebase::rules) fn compile(options: Options) -> Result<CompiledOptions> {
    let mut script_dirs = Vec::with_capacity(options.script_dirs.len());
    let mut seen_dirs = BTreeSet::new();
    for (index, raw) in options.script_dirs.into_iter().enumerate() {
        let dir = normalize_script_dir(&raw, index)?;
        ensure!(
            seen_dirs.insert(dir.clone()),
            "{RULE_ID}: options.scriptDirs[{index}] `{dir}` is listed more than once"
        );
        script_dirs.push(dir);
    }

    for (index, pattern) in options.include.iter().enumerate() {
        ensure!(
            !pattern.trim().is_empty(),
            "{RULE_ID}: options.include[{index}] must be a non-empty glob"
        );
        if let Err(error) = GlobBuilder::new(pattern).build() {
            bail!("{RULE_ID}: invalid glob in options.include[{index}] `{pattern}`: {error}");
        }
    }

    let mut events = Vec::new();
    let mut seen_events = BTreeSet::new();
    for (index, raw) in options.events.into_iter().enumerate() {
        let event = raw.trim();
        ensure!(
            PATH_FILTER_EVENTS.contains(&event),
            "{RULE_ID}: options.events[{index}] `{raw}` must be one of push, pull_request, pull_request_target"
        );
        ensure!(
            seen_events.insert(event.to_string()),
            "{RULE_ID}: options.events[{index}] `{event}` is listed more than once"
        );
        events.push(event.to_string());
    }
    if events.is_empty() {
        events.extend(PATH_FILTER_EVENTS.iter().map(|event| (*event).to_string()));
    }

    Ok(CompiledOptions {
        script_dirs,
        include: options.include,
        events,
    })
}

fn normalize_script_dir(raw: &str, index: usize) -> Result<String> {
    let trimmed = raw.trim();
    ensure!(
        !trimmed.is_empty(),
        "{RULE_ID}: options.scriptDirs[{index}] must be a non-empty repository-relative directory"
    );
    ensure!(
        !trimmed.starts_with('/') && !trimmed.contains('\\'),
        "{RULE_ID}: options.scriptDirs[{index}] `{raw}` must be a repository-relative forward-slash directory"
    );
    ensure!(
        !trimmed.contains(['*', '?', '[', ']', '{', '}']),
        "{RULE_ID}: options.scriptDirs[{index}] `{raw}` must be a literal directory path without glob characters"
    );
    let mut segments = Vec::new();
    for segment in trimmed.split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        ensure!(
            segment != "..",
            "{RULE_ID}: options.scriptDirs[{index}] `{raw}` must not contain `..`"
        );
        segments.push(segment);
    }
    ensure!(
        !segments.is_empty(),
        "{RULE_ID}: options.scriptDirs[{index}] `{raw}` must name a subdirectory inside the repository"
    );
    Ok(segments.join("/"))
}

pub(super) fn matches_script_dir(path: &str, script_dirs: &[String]) -> bool {
    script_dirs.iter().any(|dir| {
        path.strip_prefix(dir.as_str())
            .is_some_and(|rest| rest.starts_with('/') && rest.len() > 1)
    })
}

pub(super) fn prefixes_script_dir(path: &str, script_dirs: &[String]) -> bool {
    script_dirs.iter().any(|dir| {
        path == dir
            || path
                .strip_prefix(dir.as_str())
                .is_some_and(|rest| rest.starts_with('/'))
    })
}
