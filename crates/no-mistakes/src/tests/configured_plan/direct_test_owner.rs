use super::finalize::{attach_targets, sorted_selected_tests, sorted_warnings};
use super::vitest_setup_fallback;
use super::{relative_path, SelectedTest, TestFramework, TestPlan, TestPlanGroupResult};
use crate::tests::configured_plan_candidates::merge_selected;
use crate::tests::{push_resource_diagnostics, warning_key, Warning, WarningKey};
use anyhow::Result;
use no_mistakes::codebase::dependencies::graph::NodeId;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::PathBuf;

/// Select only framework-owned changed tests and framework-owned tests that
/// directly depend on a changed file. This is intentionally separate from
/// configured planning: its meaning is a one-edge owner query, so group
/// ordering, limits, samples, and fallback policy must not affect it.
pub(crate) fn generate_direct_test_owner_plan_with_prepared(
    framework: TestFramework,
    prepared: &super::super::prepared_plan::PreparedTestPlanRequest,
) -> Result<TestPlan> {
    // Both operations borrow the same request-scoped inventory, sources, fact
    // pass, and graph configuration. They have no ordering dependency, so run
    // them concurrently and merge deterministically below.
    let (graph, discovered_tests) =
        rayon::join(|| prepared.graph(), || prepared.discover_tests(framework));
    let graph = graph?;
    let discovered_tests = discovered_tests?;
    let root = &prepared.root;
    let discovered_set: HashSet<PathBuf> = discovered_tests.tests.iter().cloned().collect();
    let mut selected = BTreeMap::<PathBuf, SelectedTest>::new();

    // Include deleted sources too: their phantom node remains in the prepared
    // graph specifically so a direct reverse owner edge stays queryable.
    let changed_files = prepared
        .changed_files
        .iter()
        .chain(prepared.collected.deleted.iter())
        .collect::<BTreeSet<_>>();
    for changed in changed_files {
        let changed_rel = relative_path(root, changed);
        if discovered_set.contains(changed) {
            insert_selection(
                &mut selected,
                changed.clone(),
                SelectedTest {
                    test_file: changed_rel.clone(),
                    confidence: crate::tests::Confidence::High,
                    targets: Vec::new(),
                    reasons: vec![crate::tests::ImpactReason {
                        changed_file: changed_rel.clone(),
                        path: vec![changed_rel.clone()],
                        via: vec!["self".to_string()],
                        via_details: Vec::new(),
                    }],
                },
            );
        }

        let changed_node = NodeId::file(changed.clone());
        for (dependent, edge) in graph
            .dependents_of_node(&changed_node)
            .into_iter()
            .flatten()
        {
            let NodeId::File(test_path) = dependent else {
                continue;
            };
            if !discovered_set.contains(test_path.as_ref()) {
                continue;
            }
            let test_rel = relative_path(root, test_path);
            insert_selection(
                &mut selected,
                test_path.to_path_buf(),
                SelectedTest {
                    test_file: test_rel.clone(),
                    confidence: crate::tests::plan::path_confidence(&[*edge]),
                    targets: Vec::new(),
                    reasons: vec![crate::tests::ImpactReason {
                        changed_file: changed_rel.clone(),
                        path: vec![changed_rel.clone(), test_rel],
                        via: vec![crate::tests::plan::impact_reason_label(*edge).to_string()],
                        via_details: vec![crate::tests::plan::resource_edge_detail(
                            graph,
                            dependent,
                            &changed_node,
                            *edge,
                            root,
                        )],
                    }],
                },
            );
        }
    }

    let selected_tests = sorted_selected_tests(selected);
    let mut plan = TestPlan {
        changed_files: Vec::new(),
        groups: vec![TestPlanGroupResult {
            r#type: "direct-test-owner".to_string(),
            selected: selected_tests
                .iter()
                .map(|test| test.test_file.clone())
                .collect(),
            remaining: discovered_tests
                .tests
                .len()
                .saturating_sub(selected_tests.len()),
            limit: None,
        }],
        selected_tests,
        warnings: direct_owner_warnings(framework, prepared, graph),
        fallback_triggered: false,
        fallback_reason: None,
    };
    attach_targets(&mut plan, root, &discovered_tests);
    Ok(plan)
}

/// Reuse the prepared graph's canonical resource diagnostics. Direct-owner
/// plans intentionally skip configured-plan candidate traversal, but dynamic
/// resource calls in changed files are still incomplete reverse-owner facts.
/// The graph and its fact pass have already been initialized above.
fn direct_owner_warnings(
    framework: TestFramework,
    prepared: &super::super::prepared_plan::PreparedTestPlanRequest,
    graph: &no_mistakes::codebase::dependencies::graph::DepGraph,
) -> Vec<Warning> {
    let mut warnings = prepared.tsconfig_warnings();
    let mut warnings_seen: HashSet<WarningKey> = warnings.iter().map(warning_key).collect();
    for changed in prepared
        .changed_files
        .iter()
        .chain(prepared.collected.deleted.iter())
    {
        push_resource_diagnostics(
            graph,
            &prepared.root,
            changed,
            &mut warnings,
            &mut warnings_seen,
        );
    }
    warnings.extend(vitest_setup_fallback::framework_warnings(
        framework,
        &prepared.root,
        prepared.vitest_projects(),
    ));
    sorted_warnings(warnings)
}

fn insert_selection(
    selected: &mut BTreeMap<PathBuf, SelectedTest>,
    path: PathBuf,
    next: SelectedTest,
) {
    selected
        .entry(path)
        .and_modify(|existing| merge_selected(existing, &next))
        .or_insert(next);
}
