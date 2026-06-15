//! Evaluate whether a changed file triggers a workflow via its `on:` path
//! filters.
//!
//! Matching approximates GitHub's filter-pattern semantics using `globset`
//! with `literal_separator` enabled, so `*` does not cross `/` while `**`
//! does. Ordered `!` negations within `paths` follow gitignore-style
//! last-match-wins. Rarely-used extglob forms (`+()`, `?()`) are not supported.

use super::model::{PathFilter, Workflow};
use globset::GlobBuilder;
use serde::Serialize;

/// Result of evaluating a workflow's triggers against a single changed file.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TriggerMatch {
    /// The file matched an explicit `paths` filter (or escaped `paths-ignore`).
    Matched,
    /// A path-filterable event runs on any change (no `paths`/`paths-ignore`).
    Always,
    /// Path-filterable events exist but the file matched none of them.
    NotMatched,
    /// No path-filterable event (`push`/`pull_request`); not triggered by files.
    NoPathEvents,
}

/// A filter pattern that selected a changed file.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MatchedFilter {
    /// Event name (`push`, `pull_request`, …).
    pub event: String,
    /// The pattern (or a synthetic marker for `paths-ignore` escapes).
    pub pattern: String,
}

/// Evaluate a workflow against one changed file (repo-relative, slash path).
pub fn evaluate_trigger(
    workflow: &Workflow,
    changed_rel: &str,
) -> (TriggerMatch, Vec<MatchedFilter>) {
    if workflow.triggers.events.is_empty() {
        return (TriggerMatch::NoPathEvents, Vec::new());
    }

    let mut matched_filters = Vec::new();
    let mut any_always = false;

    // Iterate events deterministically (BTreeMap is already ordered).
    for (event, filter) in &workflow.triggers.events {
        match evaluate_event(filter, changed_rel) {
            EventOutcome::Always => any_always = true,
            EventOutcome::Matched(patterns) => {
                for pattern in patterns {
                    matched_filters.push(MatchedFilter {
                        event: event.clone(),
                        pattern,
                    });
                }
            }
            EventOutcome::NotMatched => {}
        }
    }

    if !matched_filters.is_empty() {
        (TriggerMatch::Matched, matched_filters)
    } else if any_always {
        (TriggerMatch::Always, Vec::new())
    } else {
        (TriggerMatch::NotMatched, Vec::new())
    }
}

enum EventOutcome {
    Always,
    Matched(Vec<String>),
    NotMatched,
}

fn evaluate_event(filter: &PathFilter, path: &str) -> EventOutcome {
    if filter.is_unconstrained() {
        return EventOutcome::Always;
    }
    if !filter.paths.is_empty() {
        let matched = matching_patterns(&filter.paths, path);
        if selected_by(&filter.paths, path) {
            return EventOutcome::Matched(matched);
        }
        return EventOutcome::NotMatched;
    }
    // paths-ignore only: the file triggers the event unless it is ignored.
    if !selected_by(&filter.paths_ignore, path) {
        return EventOutcome::Matched(vec!["(not ignored)".to_string()]);
    }
    EventOutcome::NotMatched
}

/// gitignore-style ordered evaluation: positive patterns select a path, `!`
/// patterns deselect it, last match wins. Returns whether `path` is selected.
fn selected_by(patterns: &[String], path: &str) -> bool {
    let mut selected = false;
    for pattern in patterns {
        let (negate, glob) = split_negation(pattern);
        if glob_matches(glob, path) {
            selected = !negate;
        }
    }
    selected
}

/// The positive patterns (ignoring `!` negations) that match `path`, for
/// reporting which filter caused the match.
fn matching_patterns(patterns: &[String], path: &str) -> Vec<String> {
    patterns
        .iter()
        .filter(|pattern| {
            let (negate, glob) = split_negation(pattern);
            !negate && glob_matches(glob, path)
        })
        .cloned()
        .collect()
}

fn split_negation(pattern: &str) -> (bool, &str) {
    match pattern.strip_prefix('!') {
        Some(rest) => (true, rest),
        None => (false, pattern),
    }
}

fn glob_matches(pattern: &str, path: &str) -> bool {
    match GlobBuilder::new(pattern).literal_separator(true).build() {
        Ok(glob) => glob.compile_matcher().is_match(path),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests;
