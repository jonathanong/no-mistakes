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
use no_mistakes::codebase::ts_source::byte_offset_to_line;
use no_mistakes::playwright::playwright_tests::{
    build_call_context_index, CallContext, TestOccurrenceScope, TestPolicy, TestStatus,
};
use rayon::prelude::*;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

/// `(binding, kind, name)` triple identifying a removed (or test-dependent)
/// queue identifier. `binding` is `Some` only for `.add(...)` jobs, where it
/// scopes the job to a specific queue instance (`emailQueue` vs
/// `billingQueue`). Worker and factory shapes don't carry a binding.
type QueueIdent = (Option<String>, String, String);

// Route literals: navigation helpers and JSX route attributes only.
// Verb calls (`app.get`, `client.get`, …) are owned by the HTTP arm so a
// removed `client.get('/dashboard')` is not double-counted as a route
// rename. The `href=`/`to=` arm covers `<Link href="/old">`,
// `<a href="/old">`, and the React Router `to=` shape, which the
// dependency graph records as `RouteRef`s.
const ROUTE_LITERAL_PATTERN: &str = concat!(
    r#"(?:\b(?:redirect|push|replace|prefetch|navigate|goto)"#,
    r#"(?:<[^>]+>)?\s*\(\s*['"`](?P<value>/[^'"`{}$\s]*)['"`]"#,
    r#"|\b(?:href|to)\s*=\s*['"`](?P<jsx>/[^'"`{}$\s]*)['"`])"#,
);

// HTTP call literals. The optional `<...>` allows typed clients like
// `client.get<User>("/api/users")` to match the same way the AST
// extractor sees them. `route` covers the chained backend shape
// `app.route('/api/x').get(handler)` — the dependency graph treats those
// the same way `app.get` would, so the rename hint follows suit.
const HTTP_LITERAL_PATTERN: &str = concat!(
    r#"(?:\bfetch|\.\s*(?:get|post|put|patch|delete|head|options|route))"#,
    r#"(?:<[^>]+>)?\s*\(\s*['"`](?P<value>/[^'"`{}$\s]*)['"`]"#,
);

mod queue_scan;
use queue_scan::{scan_addbulk_names_kinded, scan_queue_add_calls, scan_queue_defining_shapes};

pub(super) fn removed_route_paths_per_file(
    diff_files: &[DiffFile],
) -> BTreeMap<PathBuf, Vec<String>> {
    scan_string_domain(diff_files, ROUTE_LITERAL_PATTERN, &["value", "jsx"])
}

pub(super) fn removed_http_paths_per_file(
    diff_files: &[DiffFile],
) -> BTreeMap<PathBuf, Vec<String>> {
    scan_string_domain(diff_files, HTTP_LITERAL_PATTERN, &["value"])
}

pub(super) fn removed_queue_jobs_per_file(
    diff_files: &[DiffFile],
) -> BTreeMap<PathBuf, Vec<QueueIdent>> {
    // Two queue identifier kinds:
    //   - JOB pairs carry the binding of the `.add(...)` call (so two queues
    //     that happen to share a job name don't get conflated).
    //   - QUEUE entries (worker / factory) carry no binding.
    let jobs = scan_queue_add_calls(diff_files);
    let bulk = scan_addbulk_names_kinded(diff_files);
    let queues = scan_queue_defining_shapes(diff_files);
    merge_kinded_per_file_maps(vec![jobs, bulk, queues])
}

pub(super) const QUEUE_KIND_JOB: &str = "job";
pub(super) const QUEUE_KIND_QUEUE: &str = "queue";

fn merge_kinded_per_file_maps(
    maps: Vec<BTreeMap<PathBuf, Vec<QueueIdent>>>,
) -> BTreeMap<PathBuf, Vec<QueueIdent>> {
    let mut out: BTreeMap<PathBuf, Vec<QueueIdent>> = BTreeMap::new();
    for map in maps {
        for (path, values) in map {
            out.entry(path).or_default().extend(values);
        }
    }
    for values in out.values_mut() {
        values.sort();
        values.dedup();
    }
    out
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
    // Two passes: the single-line scan over `-` lines alone catches the
    // common case without picking up values that only appear on context.
    // The "removed ∪ context" scan over the hunk's source order catches
    // multi-line statements like `router.push(\n  "/old"\n);` where the
    // literal sits on a `-` line but the surrounding call shape lives on
    // context — preserving the relative order is what lets the regex see
    // the call token *before* the literal. Anything that surfaces only
    // through the context-augmented pass is verified to NOT also appear in
    // a context-only scan, so a value that lives purely on an unchanged
    // context line (e.g. inside a comment block) doesn't get falsely
    // reported as removed.
    let removed_only = scan_lines(re, capture_names, &df.removed_lines);
    let context_only: HashSet<String> = scan_lines(re, capture_names, &df.context_lines)
        .into_iter()
        .collect();
    let removed_with_ctx = scan_lines(re, capture_names, &df.removed_with_context_in_order());
    // Two different "still-present" sets:
    //   - For the `-`-only pass, only `+` lines count as still-present. A
    //     value matched on a `-` line but ALSO sitting on an unchanged
    //     context line is real removal — the diff actually changed it (the
    //     pure-context occurrence is, e.g., a comment).
    //   - For the multi-line pass, the symmetric scan over added∪context
    //     suppresses values whose span on the post-diff side is still
    //     reachable through context.
    let added_only: HashSet<String> = scan_lines(re, capture_names, &df.added_lines)
        .into_iter()
        .collect();
    let added_with_ctx: HashSet<String> =
        scan_lines(re, capture_names, &df.added_with_context_in_order())
            .into_iter()
            .collect();
    let mut truly_removed: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for value in removed_only {
        if added_only.contains(&value) {
            continue;
        }
        if seen.insert(value.clone()) {
            truly_removed.push(value);
        }
    }
    for value in removed_with_ctx {
        if context_only.contains(&value) {
            continue;
        }
        if added_with_ctx.contains(&value) {
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

/// Playwright configuration needed to make the route reverse index
/// agree with what the normal `analyze_test_occurrences` would index:
/// the `navigation_helpers` allow `navigateTo(page, "/x")` and other
/// configured wrappers to register as URL occurrences. `base_urls`
/// carries the project's playwright `baseURL` settings so absolute
/// test URLs (`page.goto('https://app.example.com/dashboard')`) can be
/// stripped to the rooted form (`/dashboard`) the diff-side scanner
/// records.
#[derive(Default)]
pub(super) struct RouteDependentConfig {
    pub navigation_helpers: Vec<String>,
    pub base_urls: Vec<String>,
}

#[derive(Default)]
struct PerTestExtraction {
    routes: Vec<String>,
    queues: Vec<QueueIdent>,
    http: Vec<String>,
}

#[derive(Default)]
pub(super) struct StringDomainDependents {
    pub routes: HashMap<String, Vec<PathBuf>>,
    pub queues: HashMap<QueueIdent, Vec<PathBuf>>,
    pub http: HashMap<String, Vec<PathBuf>>,
}

/// Run a single parallel pass over `all_tests`, parsing each file at most
/// once and extracting facts for every domain whose flag is set in
/// `domains`. Returns the merged reverse indexes. Domains with their flag
/// off come back empty.
pub(super) fn build_dependents(
    all_tests: &[PathBuf],
    domains: DependentDomains,
    route_config: &RouteDependentConfig,
) -> StringDomainDependents {
    if !domains.routes && !domains.queues && !domains.http {
        return StringDomainDependents::default();
    }
    let per_test: Vec<(PathBuf, PerTestExtraction)> = all_tests
        .par_iter()
        .map(|path| (path.clone(), extract_for_test(path, &domains, route_config)))
        .collect();
    let mut routes: Vec<(PathBuf, Vec<String>)> = Vec::with_capacity(per_test.len());
    let mut queues: Vec<(PathBuf, Vec<QueueIdent>)> = Vec::with_capacity(per_test.len());
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
        queues: merge_tuple_dependents(queues),
        http: merge_string_dependents(http),
    }
}

fn extract_for_test(
    path: &Path,
    domains: &DependentDomains,
    route_config: &RouteDependentConfig,
) -> PerTestExtraction {
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
            // Use the occurrence variant so we can mirror what
            // `analyze_test_occurrences` does: drop URLs whose enclosing
            // `test.skip(...)` / `TeardownHook` would have been filtered
            // out of `Edge::Route`. Pass through the project's configured
            // navigation helpers so `navigateTo(page, "/x")` (and similar
            // wrappers) participate in the reverse index — the normal
            // analyzer already does this.
            let policy = TestPolicy::default();
            let mut urls: Vec<String> =
                no_mistakes::playwright::playwright_urls::extract_playwright_url_occurrences_from_program(
                    program, source, &route_config.navigation_helpers,
                )
                .into_iter()
                .filter(|occ| policy.allows(occ.status))
                .filter(|occ| occ.scope != TestOccurrenceScope::TeardownHook)
                .filter_map(|occ| normalize_route_url(&occ.value, &route_config.base_urls))
                .collect();
            urls.sort();
            urls.dedup();
            out.routes = urls;
        }
        // Both HTTP and queue branches reuse the same per-call line→context
        // map so a single AST walk picks up the same `TestPolicy` and
        // `TeardownHook` filter the selector and route reverse indexes apply
        // — keeping spec-internal calls in `test.skip(...)` / teardown out of
        // the dependent set without parsing the program twice per test file.
        let needs_context = domains.queues || domains.http;
        let context_index = if needs_context {
            build_call_context_index(program, source)
        } else {
            HashMap::new()
        };
        let policy = TestPolicy::default();
        if domains.queues {
            let usage =
                no_mistakes::codebase::ts_queues::usage::extract_queue_usage_from_program(
                    program, source,
                );
            let mut pairs: Vec<QueueIdent> = Vec::new();
            for enqueue in &usage.enqueue_calls {
                if !line_allowed(&context_index, enqueue.line, policy) {
                    continue;
                }
                if let Some(job) = enqueue.job.as_ref() {
                    let binding = if enqueue.binding.is_empty() {
                        None
                    } else {
                        Some(enqueue.binding.clone())
                    };
                    pairs.push((binding, QUEUE_KIND_JOB.to_string(), job.clone()));
                }
            }
            for worker in &usage.worker_declarations {
                if !line_allowed(&context_index, worker.line, policy) {
                    continue;
                }
                if let Some(name) = worker.queue_name.as_ref() {
                    pairs.push((None, QUEUE_KIND_QUEUE.to_string(), name.clone()));
                }
            }
            // Factories: `createQueue('x')` and `new Queue('x')` are not part
            // of `extract_queue_usage_from_program`'s output today, so a
            // targeted regex picks them up directly from the test source and
            // shares the same line→context filter as the AST-derived calls.
            if let Ok(factory_re) = regex::Regex::new(
                r#"(?:\bcreateQueue\s*\(\s*['"`](?P<create>[^'"`{}$\s]+)['"`]|\bnew\s+Queue\s*\(\s*['"`](?P<queue>[^'"`{}$\s]+)['"`])"#,
            ) {
                for caps in factory_re.captures_iter(source) {
                    let Some(full) = caps.get(0) else { continue };
                    let line = byte_offset_to_line(source, full.start());
                    if !line_allowed(&context_index, line, policy) {
                        continue;
                    }
                    if let Some(m) = caps.name("create").or_else(|| caps.name("queue")) {
                        pairs.push((None, QUEUE_KIND_QUEUE.to_string(), m.as_str().to_string()));
                    }
                }
            }
            pairs.sort();
            pairs.dedup();
            out.queues = pairs;
        }
        if domains.http {
            // No backend-prefix filter: a test's `fetch('/api/foo')` should
            // match a backend that removed that exact path even when the
            // project has not declared backend prefixes via config.
            let calls =
                no_mistakes::codebase::ts_http_calls::extract_http_calls_from_program(
                    program, source, &[""],
                );
            let mut paths: Vec<String> = calls
                .into_iter()
                .filter(|c| line_allowed(&context_index, c.line, policy))
                .map(|c| c.path)
                .collect();
            paths.sort();
            paths.dedup();
            out.http = paths;
        }
        out
    })
    .unwrap_or_default()
}

/// Map a Playwright-test URL to the rooted form (`/path`) the diff scanner
/// records on the source side. Rooted references pass through unchanged; an
/// absolute `http(s)://host/path` URL is only kept when `host` matches one
/// of the project's playwright `baseURL` entries, so an unrelated external
/// site that happens to share the same path does not get conflated with the
/// app's own routes. Delegates to the existing playwright analyzer helper
/// so the two normalizations cannot drift.
fn normalize_route_url(raw: &str, base_urls: &[String]) -> Option<String> {
    no_mistakes::playwright::url::normalize_url(raw, base_urls)
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

fn merge_tuple_dependents(
    per_test: Vec<(PathBuf, Vec<QueueIdent>)>,
) -> HashMap<QueueIdent, Vec<PathBuf>> {
    let mut out: HashMap<QueueIdent, Vec<PathBuf>> = HashMap::new();
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

fn line_allowed(context_index: &HashMap<u32, CallContext>, line: u32, policy: TestPolicy) -> bool {
    let ctx = context_index.get(&line).copied();
    let status = ctx.map(|c| c.status).unwrap_or(TestStatus::Active);
    let scope = ctx.map(|c| c.scope).unwrap_or(TestOccurrenceScope::File);
    policy.allows(status) && scope != TestOccurrenceScope::TeardownHook
}
