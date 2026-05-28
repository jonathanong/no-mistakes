//! Best-effort per-domain scanners and reverse-index builders that power the
//! non-selector entries of [`super::super::configured_plan_candidates::CoverageHints`].
//!
//! Each domain (route paths, queue jobs, HTTP calls) follows the same shape:
//! - a regex scanner over a diff `DiffFile`'s `removed_lines` recovers raw
//!   identifier candidates, with values still present on `added_lines`
//!   subtracted so reformatting moves do not register as removals;
//! - a parallel pass over the discovered test set produces a reverse map
//!   `identifier -> Vec<test_file>` using the same AST extractors the
//!   playwright analyzer uses, so the dependent side stays AST-precise.
//!
//! Matching across the two sides is exact-string for v1; pattern-aware
//! matching (e.g. `/users/:id` vs `/users/123`) is a known follow-up.

use super::super::diff_parser::DiffFile;
use no_mistakes::ast::with_program;
use rayon::prelude::*;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

const ROUTE_LITERAL_PATTERN: &str = concat!(
    r#"(?:\b(?:fetch|redirect|push|replace|prefetch|navigate|goto)|"#,
    r#"\.(?:get|post|put|patch|delete|head|options|all|use))"#,
    r#"\s*\(\s*['"`](?P<value>/[^'"`{}$\s]*)['"`]"#,
);

const HTTP_LITERAL_PATTERN: &str = concat!(
    r#"(?:\bfetch|\.\s*(?:get|post|put|patch|delete|head|options))"#,
    r#"\s*\(\s*['"`](?P<value>/[^'"`{}$\s]*)['"`]"#,
);

const QUEUE_JOB_PATTERN: &str = concat!(
    r#"(?:\.addBulk\s*\(\s*\[\s*\{\s*name\s*:\s*['"`](?P<bulk>[^'"`{}$\s]+)['"`]|"#,
    r#"\.add\s*\(\s*['"`](?P<add>[^'"`{}$\s]+)['"`]|"#,
    r#"\bnew\s+Worker\s*\(\s*['"`](?P<worker>[^'"`{}$\s]+)['"`]|"#,
    r#"\bcreateQueue\s*\(\s*['"`](?P<create>[^'"`{}$\s]+)['"`]|"#,
    r#"\bnew\s+Queue\s*\(\s*['"`](?P<queue>[^'"`{}$\s]+)['"`])"#,
);

pub(super) fn removed_route_paths_per_file(
    diff_files: &[DiffFile],
) -> BTreeMap<PathBuf, Vec<String>> {
    scan_string_domain(diff_files, ROUTE_LITERAL_PATTERN, &["value"])
}

pub(super) fn removed_http_paths_per_file(
    diff_files: &[DiffFile],
) -> BTreeMap<PathBuf, Vec<String>> {
    scan_string_domain(diff_files, HTTP_LITERAL_PATTERN, &["value"])
}

pub(super) fn removed_queue_jobs_per_file(
    diff_files: &[DiffFile],
) -> BTreeMap<PathBuf, Vec<String>> {
    scan_string_domain(
        diff_files,
        QUEUE_JOB_PATTERN,
        &["bulk", "add", "worker", "create", "queue"],
    )
}

fn scan_string_domain(
    diff_files: &[DiffFile],
    pattern: &str,
    capture_names: &[&str],
) -> BTreeMap<PathBuf, Vec<String>> {
    let mut out: BTreeMap<PathBuf, Vec<String>> = BTreeMap::new();
    if diff_files.is_empty() {
        return out;
    }
    let Ok(re) = regex::Regex::new(pattern) else {
        return out;
    };
    let per_file: Vec<(PathBuf, Vec<String>)> = diff_files
        .par_iter()
        .filter_map(|df| truly_removed_strings(&re, capture_names, df))
        .collect();
    for (key, values) in per_file {
        out.entry(key).or_default().extend(values);
    }
    for values in out.values_mut() {
        values.sort();
        values.dedup();
    }
    out
}

fn truly_removed_strings(
    re: &regex::Regex,
    capture_names: &[&str],
    df: &DiffFile,
) -> Option<(PathBuf, Vec<String>)> {
    if df.removed_lines.is_empty() {
        return None;
    }
    let removed = scan_lines(re, capture_names, &df.removed_lines);
    if removed.is_empty() {
        return None;
    }
    let added: HashSet<String> = scan_lines(re, capture_names, &df.added_lines)
        .into_iter()
        .collect();
    let mut truly_removed: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for value in removed {
        if added.contains(&value) {
            continue;
        }
        if seen.insert(value.clone()) {
            truly_removed.push(value);
        }
    }
    if truly_removed.is_empty() {
        return None;
    }
    Some((df.path.clone(), truly_removed))
}

fn scan_lines(re: &regex::Regex, capture_names: &[&str], lines: &[String]) -> Vec<String> {
    // Join the hunk's lines so formatter-wrapped statements like
    // `router.push(\n  "/x"\n)` or `addBulk([{\n  name: "x"\n}])` still
    // match — `\s*` in the patterns spans the inserted newline. This is the
    // entire reason the matcher is line-oblivious.
    let joined = lines.join("\n");
    let mut out: Vec<String> = Vec::new();
    for caps in re.captures_iter(&joined) {
        for name in capture_names {
            if let Some(m) = caps.name(name) {
                let value = m.as_str();
                if !value.is_empty() {
                    out.push(value.to_string());
                }
            }
        }
    }
    out
}

/// Per-domain "do we need to compute this reverse index?" flags. Set only
/// the domains whose corresponding `removed_*` map was non-empty so the
/// per-test pass skips unused extraction work.
#[derive(Clone, Copy, Default)]
pub(super) struct DependentDomains {
    pub routes: bool,
    pub queues: bool,
    pub http: bool,
}

#[derive(Default)]
struct PerTestExtraction {
    routes: Vec<String>,
    queues: Vec<String>,
    http: Vec<String>,
}

#[derive(Default)]
pub(super) struct StringDomainDependents {
    pub routes: HashMap<String, Vec<PathBuf>>,
    pub queues: HashMap<String, Vec<PathBuf>>,
    pub http: HashMap<String, Vec<PathBuf>>,
}

/// Run a single parallel pass over `all_tests`, parsing each file at most
/// once and extracting facts for every domain whose flag is set in
/// `domains`. Returns the merged reverse indexes. Domains with their flag
/// off come back empty.
pub(super) fn build_dependents(
    all_tests: &[PathBuf],
    domains: DependentDomains,
) -> StringDomainDependents {
    if !domains.routes && !domains.queues && !domains.http {
        return StringDomainDependents::default();
    }
    let per_test: Vec<(PathBuf, PerTestExtraction)> = all_tests
        .par_iter()
        .map(|path| (path.clone(), extract_for_test(path, &domains)))
        .collect();
    let mut routes: Vec<(PathBuf, Vec<String>)> = Vec::with_capacity(per_test.len());
    let mut queues: Vec<(PathBuf, Vec<String>)> = Vec::with_capacity(per_test.len());
    let mut http: Vec<(PathBuf, Vec<String>)> = Vec::with_capacity(per_test.len());
    for (path, ext) in per_test {
        if domains.routes {
            routes.push((path.clone(), ext.routes));
        }
        if domains.queues {
            queues.push((path.clone(), ext.queues));
        }
        if domains.http {
            http.push((path, ext.http));
        }
    }
    StringDomainDependents {
        routes: merge_string_dependents(routes),
        queues: merge_string_dependents(queues),
        http: merge_string_dependents(http),
    }
}

fn extract_for_test(path: &Path, domains: &DependentDomains) -> PerTestExtraction {
    let Ok(source) = std::fs::read_to_string(path) else {
        return PerTestExtraction::default();
    };
    // Parse each test file at most once; route/queue/http extraction all
    // operate on the same `Program`, so any combination of enabled domains
    // pays exactly one parse per file. Parse failures are swallowed
    // (best-effort augmentation).
    with_program(path, &source, |program, source| {
        let mut out = PerTestExtraction::default();
        if domains.routes {
            out.routes =
                no_mistakes::playwright::playwright_urls::extract_playwright_url_literals_from_program(
                    program, source, &[],
                );
        }
        if domains.queues {
            let usage =
                no_mistakes::codebase::ts_queues::usage::extract_queue_usage_from_program(
                    program, source,
                );
            let mut names: Vec<String> = Vec::new();
            for enqueue in &usage.enqueue_calls {
                if let Some(job) = enqueue.job.as_ref() {
                    names.push(job.clone());
                }
            }
            for worker in &usage.worker_declarations {
                if let Some(name) = worker.queue_name.as_ref() {
                    names.push(name.clone());
                }
            }
            names.sort();
            names.dedup();
            out.queues = names;
        }
        if domains.http {
            // No backend-prefix filter: a test's `fetch('/api/foo')` should
            // match a backend that removed that exact path even when the
            // project has not declared backend prefixes via config.
            let calls =
                no_mistakes::codebase::ts_http_calls::extract_http_calls_from_program(
                    program, source, &[""],
                );
            let mut paths: Vec<String> = calls.into_iter().map(|c| c.path).collect();
            paths.sort();
            paths.dedup();
            out.http = paths;
        }
        out
    })
    .unwrap_or_default()
}

fn merge_string_dependents(per_test: Vec<(PathBuf, Vec<String>)>) -> HashMap<String, Vec<PathBuf>> {
    let mut out: HashMap<String, Vec<PathBuf>> = HashMap::new();
    for (test, values) in per_test {
        for value in values {
            out.entry(value).or_default().push(test.clone());
        }
    }
    for tests in out.values_mut() {
        tests.sort();
        tests.dedup();
    }
    out
}
