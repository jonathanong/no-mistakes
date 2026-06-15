//! Evaluate whether a changed file triggers a workflow via its `on:` path
//! filters.
//!
//! Matching approximates GitHub's filter-pattern semantics using `globset`
//! with `literal_separator` enabled, so `*` does not cross `/` while `**`
//! does. Ordered `!` negations within `paths` follow gitignore-style
//! last-match-wins. Rarely-used extglob forms (`+()`, `?()`) are not supported.
//!
//! Globs are compiled once per workflow into [`CompiledTriggers`] and reused
//! across all changed files, so a multi-file query never recompiles a pattern.

use super::model::{PathFilter, Workflow};
use globset::{GlobBuilder, GlobMatcher};
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

/// A workflow's triggers with their globs pre-compiled.
pub struct CompiledTriggers {
    events: Vec<(String, CompiledFilter)>,
}

struct CompiledFilter {
    unconstrained: bool,
    has_paths: bool,
    paths: Vec<CompiledGlob>,
    paths_ignore: Vec<CompiledGlob>,
}

struct CompiledGlob {
    negate: bool,
    pattern: String,
    matcher: GlobMatcher,
}

impl CompiledTriggers {
    /// Compile every path-filterable event's globs once.
    pub fn new(workflow: &Workflow) -> Self {
        let events = workflow
            .triggers
            .events
            .iter()
            .map(|(event, filter)| (event.clone(), CompiledFilter::new(filter)))
            .collect();
        CompiledTriggers { events }
    }

    /// Evaluate the compiled triggers against one changed file.
    pub fn evaluate(&self, changed_rel: &str) -> (TriggerMatch, Vec<MatchedFilter>) {
        if self.events.is_empty() {
            return (TriggerMatch::NoPathEvents, Vec::new());
        }

        let mut matched_filters = Vec::new();
        let mut any_always = false;

        for (event, filter) in &self.events {
            match filter.evaluate(changed_rel) {
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
}

impl CompiledFilter {
    fn new(filter: &PathFilter) -> Self {
        CompiledFilter {
            unconstrained: filter.is_unconstrained(),
            has_paths: !filter.paths.is_empty(),
            paths: compile_list(&filter.paths),
            paths_ignore: compile_list(&filter.paths_ignore),
        }
    }

    fn evaluate(&self, path: &str) -> EventOutcome {
        if self.unconstrained {
            return EventOutcome::Always;
        }
        if self.has_paths {
            if selected_by(&self.paths, path) {
                return EventOutcome::Matched(matching_patterns(&self.paths, path));
            }
            return EventOutcome::NotMatched;
        }
        // paths-ignore only: the file triggers the event unless it is ignored.
        if !selected_by(&self.paths_ignore, path) {
            return EventOutcome::Matched(vec!["(not ignored)".to_string()]);
        }
        EventOutcome::NotMatched
    }
}

/// Evaluate a workflow against one changed file (repo-relative, slash path).
pub fn evaluate_trigger(
    workflow: &Workflow,
    changed_rel: &str,
) -> (TriggerMatch, Vec<MatchedFilter>) {
    CompiledTriggers::new(workflow).evaluate(changed_rel)
}

enum EventOutcome {
    Always,
    Matched(Vec<String>),
    NotMatched,
}

/// Compile each pattern once, dropping invalid globs (which never match).
fn compile_list(patterns: &[String]) -> Vec<CompiledGlob> {
    patterns
        .iter()
        .filter_map(|pattern| {
            let (negate, glob) = split_negation(pattern);
            GlobBuilder::new(glob)
                .literal_separator(true)
                .build()
                .ok()
                .map(|glob| CompiledGlob {
                    negate,
                    pattern: pattern.clone(),
                    matcher: glob.compile_matcher(),
                })
        })
        .collect()
}

/// gitignore-style ordered evaluation: positive patterns select a path, `!`
/// patterns deselect it, last match wins. Returns whether `path` is selected.
fn selected_by(globs: &[CompiledGlob], path: &str) -> bool {
    let mut selected = false;
    for glob in globs {
        if glob.matcher.is_match(path) {
            selected = !glob.negate;
        }
    }
    selected
}

/// The positive patterns (ignoring `!` negations) that match `path`, for
/// reporting which filter caused the match.
fn matching_patterns(globs: &[CompiledGlob], path: &str) -> Vec<String> {
    globs
        .iter()
        .filter(|glob| !glob.negate && glob.matcher.is_match(path))
        .map(|glob| glob.pattern.clone())
        .collect()
}

fn split_negation(pattern: &str) -> (bool, &str) {
    match pattern.strip_prefix('!') {
        Some(rest) => (true, rest),
        None => (false, pattern),
    }
}

#[cfg(test)]
mod tests;
