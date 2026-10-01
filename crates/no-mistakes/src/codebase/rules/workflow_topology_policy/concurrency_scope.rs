use regex::Regex;
use std::sync::OnceLock;

const SCOPE_ORDER: &[&str] = &[
    "pull-request",
    "ref",
    "sha",
    "run",
    "event",
    "input-resource",
];

const KNOWN_SCOPES: &[&str] = &[
    "pull-request",
    "ref",
    "sha",
    "run",
    "event",
    "input-resource",
    "fixed-resource",
];

struct Pattern {
    text: &'static str,
    scope: &'static str,
}

/// Longer patterns win. A reference matches a pattern when it is equal to the
/// pattern or continues with another dotted segment.
const PATTERNS: &[Pattern] = &[
    Pattern {
        text: "github.event.pull_request.number",
        scope: "pull-request",
    },
    Pattern {
        text: "github.event.workflow_run.pull_requests",
        scope: "pull-request",
    },
    Pattern {
        text: "github.sha",
        scope: "sha",
    },
    Pattern {
        text: "github.event.pull_request.head.sha",
        scope: "sha",
    },
    Pattern {
        text: "github.event.workflow_run.head_sha",
        scope: "sha",
    },
    Pattern {
        text: "github.ref",
        scope: "ref",
    },
    Pattern {
        text: "github.ref_name",
        scope: "ref",
    },
    Pattern {
        text: "github.event.workflow_run.head_branch",
        scope: "ref",
    },
    Pattern {
        text: "github.run_id",
        scope: "run",
    },
    Pattern {
        text: "inputs",
        scope: "input-resource",
    },
    Pattern {
        text: "github.event.inputs",
        scope: "input-resource",
    },
    Pattern {
        text: "github.event_name",
        scope: "event",
    },
    Pattern {
        text: "github.event.action",
        scope: "event",
    },
    Pattern {
        text: "github.event.label.name",
        scope: "event",
    },
    Pattern {
        text: "github.event.issue.number",
        scope: "event",
    },
    Pattern {
        text: "github.event.schedule",
        scope: "event",
    },
    Pattern {
        text: "github.event.workflow_run.event",
        scope: "event",
    },
    Pattern {
        text: "github.event.workflow_run.id",
        scope: "event",
    },
    Pattern {
        text: "github.event.workflow_run.workflow_id",
        scope: "event",
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ScopeClass {
    Ignored,
    Known(&'static str),
    Unsupported,
}

pub(super) fn classify(reference: &str) -> ScopeClass {
    if reference == "github.workflow" || reference == "github.run_attempt" {
        return ScopeClass::Ignored;
    }
    let mut best: Option<&Pattern> = None;
    for pattern in PATTERNS {
        if !matches_pattern(reference, pattern.text) {
            continue;
        }
        if best.is_none_or(|current| pattern.text.len() > current.text.len()) {
            best = Some(pattern);
        }
    }
    match best {
        Some(pattern) => ScopeClass::Known(pattern.scope),
        None => ScopeClass::Unsupported,
    }
}

pub(super) fn known_scope(name: &str) -> bool {
    KNOWN_SCOPES.contains(&name)
}

pub(super) fn sort_declared(entries: &[String]) -> Vec<String> {
    SCOPE_ORDER
        .iter()
        .filter(|name| entries.iter().any(|entry| entry == *name))
        .map(|name| (*name).to_string())
        .collect()
}

pub(super) fn actual_scope(group: &str) -> Vec<String> {
    let mut known = Vec::new();
    let mut unsupported = Vec::new();
    for reference in references(group) {
        match classify(&reference) {
            ScopeClass::Ignored => {}
            ScopeClass::Known(scope) => {
                if !known.contains(&scope) {
                    known.push(scope);
                }
            }
            ScopeClass::Unsupported => {
                if !unsupported.contains(&reference) {
                    unsupported.push(reference);
                }
            }
        }
    }
    unsupported.sort();
    let mut scopes = Vec::new();
    for name in SCOPE_ORDER {
        if known.contains(name) {
            scopes.push((*name).to_string());
        }
    }
    for reference in unsupported {
        scopes.push(format!("unsupported:{reference}"));
    }
    scopes
}

fn matches_pattern(reference: &str, pattern: &str) -> bool {
    reference == pattern
        || reference
            .strip_prefix(pattern)
            .is_some_and(|rest| rest.starts_with('.'))
}

fn references(group: &str) -> Vec<String> {
    reference_pattern()
        .find_iter(group)
        .map(|item| item.as_str().to_string())
        .collect()
}

fn reference_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r"(?-u)\b(?:github|inputs)(?:\.[A-Za-z_][\w-]*)+")
            .expect("concurrency scope reference pattern")
    })
}
