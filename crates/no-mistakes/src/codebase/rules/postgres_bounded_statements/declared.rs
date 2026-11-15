//! Bind `no-mistakes-bound` comments to checked statement anchors.
//!
//! A declaration matches the statement's outer keyword line, or the line
//! immediately above it. One statement keeps one declaration.

use crate::codebase::ts_source::bound_comments::{bound_comments, BoundComment, BoundCommentError};
use std::collections::{BTreeMap, BTreeSet};

pub(super) const MISSING_BOUND: &str = "declared bound is missing a numeric rows<=N limit; write `-- no-mistakes-bound postgres-bounded-statements rows<=100: why these rows are finite`";
pub(super) const MISSING_JUSTIFICATION: &str = "declared bound is missing a non-empty justification after rows<=N; write `-- no-mistakes-bound postgres-bounded-statements rows<=100: why these rows are finite`";
pub(super) const UNMATCHED: &str = "declared bound does not cover a checked statement; put it on the statement line or the line immediately above the outer SQL keyword";
pub(super) const DUPLICATE: &str = "a statement has more than one declared bound; keep a single rows<=N declaration with one justification";

#[derive(Debug)]
pub(super) enum Gate {
    Valid {
        line: usize,
        rows: u64,
        justification: String,
    },
    Invalid {
        line: usize,
        message: &'static str,
    },
    Blocked {
        findings: Vec<(usize, &'static str)>,
    },
}

pub(super) struct DeclarationPlan {
    gates: BTreeMap<usize, Gate>,
    unmatched: Vec<(usize, &'static str)>,
}

impl DeclarationPlan {
    pub(super) fn is_empty(&self) -> bool {
        self.gates.is_empty() && self.unmatched.is_empty()
    }

    pub(super) fn gate(&self, anchor: usize) -> Option<&Gate> {
        self.gates.get(&anchor)
    }

    pub(super) fn unmatched(&self) -> &[(usize, &'static str)] {
        &self.unmatched
    }
}

pub(super) fn plan(source: Option<&str>, anchors: &[usize]) -> DeclarationPlan {
    let Some(source) = source else {
        return DeclarationPlan {
            gates: BTreeMap::new(),
            unmatched: Vec::new(),
        };
    };
    let comments = bound_comments(source, super::RULE_ID);
    if comments.is_empty() {
        return DeclarationPlan {
            gates: BTreeMap::new(),
            unmatched: Vec::new(),
        };
    }
    let anchors: BTreeSet<usize> = anchors.iter().copied().collect();
    let mut same: BTreeMap<usize, Vec<BoundComment>> = BTreeMap::new();
    let mut previous: BTreeMap<usize, Vec<BoundComment>> = BTreeMap::new();
    let mut unmatched = Vec::new();
    for comment in comments {
        let line = comment_line(&comment);
        if anchors.contains(&line) {
            same.entry(line).or_default().push(comment);
        } else if anchors.contains(&(line + 1)) {
            previous.entry(line + 1).or_default().push(comment);
        } else {
            unmatched.push((line, unbound_message(&comment)));
        }
    }
    let mut keys: BTreeSet<usize> = same.keys().copied().collect();
    keys.extend(previous.keys().copied());
    let mut gates = BTreeMap::new();
    for anchor in keys {
        let mut decls = same.remove(&anchor).unwrap_or_default();
        decls.extend(previous.remove(&anchor).unwrap_or_default());
        gates.insert(anchor, gate_for(decls));
    }
    DeclarationPlan { gates, unmatched }
}

fn comment_line(comment: &BoundComment) -> usize {
    match comment {
        BoundComment::Valid { line, .. } | BoundComment::Invalid { line, .. } => *line as usize,
    }
}

fn gate_for(decls: Vec<BoundComment>) -> Gate {
    if decls.len() == 1 {
        return single(decls.into_iter().next().unwrap());
    }
    let mut findings: Vec<_> = decls
        .iter()
        .map(|comment| (comment_line(comment), comment_message(comment)))
        .collect();
    findings.sort_by_key(|(line, _)| *line);
    Gate::Blocked { findings }
}

fn single(comment: BoundComment) -> Gate {
    match comment {
        BoundComment::Valid {
            line,
            rows,
            justification,
        } => Gate::Valid {
            line: line as usize,
            rows,
            justification,
        },
        BoundComment::Invalid { line, error } => Gate::Invalid {
            line: line as usize,
            message: error_message(error),
        },
    }
}

fn unbound_message(comment: &BoundComment) -> &'static str {
    match comment {
        BoundComment::Valid { .. } => UNMATCHED,
        BoundComment::Invalid { error, .. } => error_message(*error),
    }
}

fn comment_message(comment: &BoundComment) -> &'static str {
    match comment {
        BoundComment::Valid { .. } => DUPLICATE,
        BoundComment::Invalid { error, .. } => error_message(*error),
    }
}

fn error_message(error: BoundCommentError) -> &'static str {
    match error {
        BoundCommentError::MissingBound => MISSING_BOUND,
        BoundCommentError::MissingJustification => MISSING_JUSTIFICATION,
    }
}

pub(super) fn stale_message(rows: u64) -> String {
    format!("declared bound rows<={rows} is stale because this statement is already provably bounded; remove the declaration")
}

#[cfg(test)]
#[path = "declared_tests.rs"]
mod tests;
