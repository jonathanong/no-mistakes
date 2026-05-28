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
    let mut out: Vec<String> = Vec::new();
    for line in lines {
        for caps in re.captures_iter(line) {
            for name in capture_names {
                if let Some(m) = caps.name(name) {
                    let value = m.as_str();
                    if !value.is_empty() {
                        out.push(value.to_string());
                    }
                }
            }
        }
    }
    out
}

pub(super) fn build_route_path_dependents(all_tests: &[PathBuf]) -> HashMap<String, Vec<PathBuf>> {
    let per_test: Vec<(PathBuf, Vec<String>)> = all_tests
        .par_iter()
        .map(|path| (path.clone(), extract_test_urls(path)))
        .collect();
    merge_string_dependents(per_test)
}

fn extract_test_urls(path: &Path) -> Vec<String> {
    let Ok(source) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    with_program(path, &source, |program, source| {
        no_mistakes::playwright::playwright_urls::extract_playwright_url_literals_from_program(
            program,
            source,
            &[],
        )
    })
    .unwrap_or_default()
}

pub(super) fn build_queue_job_dependents(all_tests: &[PathBuf]) -> HashMap<String, Vec<PathBuf>> {
    let per_test: Vec<(PathBuf, Vec<String>)> = all_tests
        .par_iter()
        .map(|path| (path.clone(), extract_queue_job_names(path)))
        .collect();
    merge_string_dependents(per_test)
}

fn extract_queue_job_names(path: &Path) -> Vec<String> {
    // The queue extractor lives in the lib's `codebase` module; reach it
    // through the public re-export so the bin can drive it.
    let Ok(source) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let usage = no_mistakes::codebase::ts_queues::usage::extract_queue_usage(&source);
    let mut out: Vec<String> = Vec::new();
    for enqueue in &usage.enqueue_calls {
        if let Some(job) = enqueue.job.as_ref() {
            out.push(job.clone());
        }
    }
    for worker in &usage.worker_declarations {
        if let Some(name) = worker.queue_name.as_ref() {
            out.push(name.clone());
        }
    }
    out.sort();
    out.dedup();
    out
}

pub(super) fn build_http_path_dependents(all_tests: &[PathBuf]) -> HashMap<String, Vec<PathBuf>> {
    let per_test: Vec<(PathBuf, Vec<String>)> = all_tests
        .par_iter()
        .map(|path| (path.clone(), extract_http_call_paths(path)))
        .collect();
    merge_string_dependents(per_test)
}

fn extract_http_call_paths(path: &Path) -> Vec<String> {
    let Ok(source) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    // No backend-prefix filter: a test's `fetch('/api/foo')` should match a
    // backend that removed that exact path even if the project has not
    // declared backend prefixes for the static-paths config knob.
    let calls = no_mistakes::codebase::ts_http_calls::extract_http_calls(&source, &[""]);
    let mut out: Vec<String> = calls.into_iter().map(|c| c.path).collect();
    out.sort();
    out.dedup();
    out
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
