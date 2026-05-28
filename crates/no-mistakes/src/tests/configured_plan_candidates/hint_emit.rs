use super::{Confidence, ImpactReason, QueueIdent, SelectedTest};
use crate::tests::plan::{impact_reason_label, path_confidence, relative_path};
use no_mistakes::codebase::dependencies::graph::EdgeKind;
use no_mistakes::codebase::test_filter::TestFileFilter;
use no_mistakes::playwright::matcher;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

#[allow(clippy::too_many_arguments)]
pub(super) fn append_removed_id_candidates<K: std::hash::Hash + Eq + Clone>(
    root: &Path,
    all_test_set: &HashSet<PathBuf>,
    test_filter: &TestFileFilter,
    used: &HashSet<String>,
    removed: &BTreeMap<PathBuf, Vec<K>>,
    dependents: &HashMap<K, Vec<PathBuf>>,
    edge: EdgeKind,
    selected: &mut BTreeMap<String, SelectedTest>,
) {
    let confidence = path_confidence(&[edge]);
    let via_label = impact_reason_label(edge).to_string();
    // Iterate the hint map directly so deleted source files (which are
    // filtered out of `changed_files` because they no longer exist on disk)
    // still contribute their removed identifiers.
    for (changed_path, ids) in removed {
        if changed_path_is_test(changed_path, all_test_set, test_filter, root) {
            continue;
        }
        let rel_changed = relative_path(root, changed_path);
        let mut tests_for_changed: HashSet<PathBuf> = HashSet::new();
        for id in ids {
            let Some(tests) = dependents.get(id) else {
                continue;
            };
            collect_dependent_tests(tests, changed_path, all_test_set, &mut tests_for_changed);
        }
        emit_hint_reasons(
            root,
            used,
            &rel_changed,
            tests_for_changed,
            confidence,
            &via_label,
            selected,
        );
    }
}

/// Queue-specific wrapper that forwards to the generic exact-key lookup with
/// the `EdgeKind::QueueEnqueue` confidence/via-label. Kept as a dedicated
/// entry point so the call sites under the dependents/queue branches read
/// the same as the route/HTTP variants without inlining `EdgeKind` choices
/// at each call site.
#[allow(clippy::too_many_arguments)]
pub(super) fn append_queue_hint_candidates(
    root: &Path,
    all_test_set: &HashSet<PathBuf>,
    test_filter: &TestFileFilter,
    used: &HashSet<String>,
    removed: &BTreeMap<PathBuf, Vec<QueueIdent>>,
    dependents: &HashMap<QueueIdent, Vec<PathBuf>>,
    selected: &mut BTreeMap<String, SelectedTest>,
) {
    append_removed_id_candidates(
        root,
        all_test_set,
        test_filter,
        used,
        removed,
        dependents,
        EdgeKind::QueueEnqueue,
        selected,
    );
}

/// Route-specific variant that uses `matcher::matches` to line up parametric
/// source patterns (`/users/:id`) with concrete test references
/// (`/users/123`). All other edge kinds key on exact-string identifiers and
/// stay on the cheaper `HashMap::get` path in `append_removed_id_candidates`.
pub(super) fn append_route_hint_candidates(
    root: &Path,
    all_test_set: &HashSet<PathBuf>,
    test_filter: &TestFileFilter,
    used: &HashSet<String>,
    removed: &BTreeMap<PathBuf, Vec<String>>,
    dependents: &HashMap<String, Vec<PathBuf>>,
    selected: &mut BTreeMap<String, SelectedTest>,
) {
    let confidence = path_confidence(&[EdgeKind::RouteTest]);
    let via_label = impact_reason_label(EdgeKind::RouteTest).to_string();
    for (changed_path, patterns) in removed {
        if changed_path_is_test(changed_path, all_test_set, test_filter, root) {
            continue;
        }
        let rel_changed = relative_path(root, changed_path);
        let mut tests_for_changed: HashSet<PathBuf> = HashSet::new();
        for pattern in patterns {
            if let Some(tests) = dependents.get(pattern) {
                collect_dependent_tests(tests, changed_path, all_test_set, &mut tests_for_changed);
            }
            // Pattern-aware: a `/users/:id` removal should also catch tests
            // that navigate to a concrete reference like `/users/123`. Only
            // walk the reverse index when the source pattern contains a
            // parametric segment, to keep the iteration cost bounded for
            // plain literals.
            if has_pattern_segment(pattern) {
                for (reference, tests) in dependents {
                    if reference == pattern {
                        continue;
                    }
                    if matcher::matches(reference, pattern) {
                        collect_dependent_tests(
                            tests,
                            changed_path,
                            all_test_set,
                            &mut tests_for_changed,
                        );
                    }
                }
            }
        }
        emit_hint_reasons(
            root,
            used,
            &rel_changed,
            tests_for_changed,
            confidence,
            &via_label,
            selected,
        );
    }
}

fn has_pattern_segment(pattern: &str) -> bool {
    pattern
        .split('/')
        .any(|segment| segment.starts_with(':') || segment == "*" || segment == "**")
}

fn changed_path_is_test(
    changed_path: &Path,
    all_test_set: &HashSet<PathBuf>,
    test_filter: &TestFileFilter,
    root: &Path,
) -> bool {
    // Skip hints sourced from a test file itself. A test that edits its own
    // `page.goto('/old')` -> `page.goto('/new')` would otherwise surface every
    // OTHER spec that still uses `/old`, even though no app/source identifier
    // was removed. `all_test_set` only contains currently-discovered tests,
    // so a *deleted* spec is missed here; fall back to the project's test
    // glob filter (which understands both configured suites and the default
    // `__tests__`/`*.test.*`/`*.spec.*` shape) so removed identifiers in a
    // deleted spec do not get treated as a source removal.
    if all_test_set.contains(changed_path) {
        return true;
    }
    test_filter.is_match(root, changed_path)
}

fn collect_dependent_tests(
    tests: &[PathBuf],
    changed_path: &Path,
    all_test_set: &HashSet<PathBuf>,
    out: &mut HashSet<PathBuf>,
) {
    for test in tests {
        if test == changed_path {
            continue;
        }
        if !all_test_set.contains(test) {
            continue;
        }
        out.insert(test.clone());
    }
}

fn emit_hint_reasons(
    root: &Path,
    used: &HashSet<String>,
    rel_changed: &str,
    tests_for_changed: HashSet<PathBuf>,
    confidence: Confidence,
    via_label: &str,
    selected: &mut BTreeMap<String, SelectedTest>,
) {
    let mut sorted: Vec<PathBuf> = tests_for_changed.into_iter().collect();
    sorted.sort();
    for test_path in sorted {
        let rel_test = relative_path(root, &test_path);
        if used.contains(&rel_test) {
            continue;
        }
        let reason = ImpactReason {
            changed_file: rel_changed.to_string(),
            path: vec![rel_changed.to_string(), rel_test.clone()],
            via: vec![via_label.to_string()],
        };
        let entry = selected
            .entry(rel_test.clone())
            .or_insert_with(|| SelectedTest {
                test_file: rel_test,
                confidence,
                reasons: Vec::new(),
            });
        if confidence > entry.confidence {
            entry.confidence = confidence;
        }
        if !entry.reasons.contains(&reason) {
            entry.reasons.push(reason);
        }
    }
}
