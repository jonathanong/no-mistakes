use crate::tests::{
    push_resource_diagnostics, via_details_from_edges, warning_key, Confidence, ImpactEdgeDetail,
    ImpactReason, PlanArgs, ResourceCallSite, SelectedTest, TestPlan, Warning, WarningKey,
};
use anyhow::Result;
use no_mistakes::codebase::dependencies::graph::{DepGraph, EdgeKind, NodeId};
use no_mistakes::codebase::test_filter::TestFileFilter;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

include!("plan_extra_inputs.rs");

#[path = "plan_vitest_setup.rs"]
mod plan_vitest_setup;

mod changed_inventory;
pub(crate) use changed_inventory::generate_plan_with_prepared;
mod dependency_seeds;
mod fallback;
mod framework;
mod global_config;
mod run;
mod trace_changed;
mod warnings;
pub(crate) use global_config::global_config_trigger;
pub(crate) use run::run;

include!("plan/generate.rs");

fn generate_plan_with_prepared_inner(
    args: &PlanArgs,
    prepared: &super::prepared_plan::PreparedTestPlanRequest,
    timing: Option<&mut crate::impacted_checks::timing::TimingTracker>,
) -> Result<TestPlan> {
    let root = &prepared.root;
    let collected = &prepared.collected;
    if let Some(framework) = args.framework {
        return framework::generate_framework_plan(args, prepared, framework, timing);
    }

    let changed_files = prepared.planning_changed_files(None);
    let deleted_files = &collected.deleted;

    if let Some(plan) = fallback::global_fallback_plan(args, prepared, &changed_files) {
        return Ok(plan);
    }

    // 3. Build graph and test filter
    let graph = prepared.graph()?;
    let test_filter = prepared.test_filter().clone();
    let all_test_files = global_config::discover_all_tests_from_prepared(prepared);
    let all_test_set: HashSet<PathBuf> = all_test_files.iter().cloned().collect();
    let mut native_test_set = all_test_set.clone();
    for framework in [super::TestFramework::Swift, super::TestFramework::Dotnet] {
        native_test_set.extend(prepared.discover_tests(framework)?.tests);
    }
    let native_semantic_seeds =
        crate::tests::configured_plan::native_semantic_seeds::native_semantic_seed_candidates(
            root,
            prepared,
            graph,
            &native_test_set,
            None,
        );

    let mut selected_map: HashMap<PathBuf, SelectedTest> = HashMap::new();
    let mut warnings = prepared.tsconfig_warnings();
    let mut warnings_seen: HashSet<WarningKey> = warnings.iter().map(warning_key).collect();

    trace_changed::trace_changed_files(
        trace_changed::TraceChangedContext {
            args,
            graph,
            test_filter: &test_filter,
            root,
            changed_files: &changed_files,
        },
        trace_changed::TraceChangedState {
            selected_map: &mut selected_map,
            warnings: &mut warnings,
            warnings_seen: &mut warnings_seen,
        },
    );

    for candidate in &native_semantic_seeds.candidates {
        selected_map
            .entry(root.join(&candidate.test_file))
            .and_modify(|existing| {
                crate::tests::configured_plan_candidates::merge_selected(existing, candidate)
            })
            .or_insert_with(|| candidate.clone());
    }

    if let Some(fallback_plan) = dependency_seeds::trace_and_fallback(
        dependency_seeds::DependencySeedContext {
            args,
            prepared,
            graph,
            test_filter: &test_filter,
            all_test_files: &all_test_files,
            native_semantic_seeds: &native_semantic_seeds,
        },
        dependency_seeds::DependencySeedState {
            selected_map: &mut selected_map,
            warnings: &mut warnings,
            warnings_seen: &mut warnings_seen,
        },
    ) {
        return Ok(fallback_plan);
    }

    warnings::extend_analysis_warnings(prepared, &mut warnings, &mut warnings_seen);

    // 5. Trace deleted files (phantom node lookup in reverse map)
    trace_deleted_files(
        deleted_files,
        graph,
        &test_filter,
        root,
        &mut selected_map,
        &mut warnings,
        &mut warnings_seen,
    );

    // 6. Trace entrypoints (file#export)
    trace_entrypoints(
        &args.entrypoints,
        &args.entrypoint_symbols,
        graph,
        &test_filter,
        root,
        &mut selected_map,
        args.include_symbols,
    )?;

    let vitest_fallback_reason = plan_vitest_setup::apply_union_fallback(
        prepared,
        root,
        &changed_files,
        deleted_files,
        &mut selected_map,
        &mut warnings,
        &mut warnings_seen,
    )?;

    let mut selected_tests: Vec<SelectedTest> = selected_map.into_values().collect();
    for test in &mut selected_tests {
        test.reasons
            .sort_by(|a, b| a.changed_file.cmp(&b.changed_file));
    }
    selected_tests.sort_by(|a, b| a.test_file.cmp(&b.test_file));
    warnings.sort_by(|a, b| {
        (&a.file, a.line, &a.r#type, &a.message).cmp(&(&b.file, b.line, &b.r#type, &b.message))
    });

    Ok(TestPlan {
        changed_files: Vec::new(),
        selected_tests,
        groups: Vec::new(),
        warnings,
        fallback_triggered: vitest_fallback_reason.is_some(),
        fallback_reason: vitest_fallback_reason,
        ..Default::default()
    })
}

include!("plan_bfs.rs");
#[cfg(test)]
include!("plan_resources_tests.rs");
#[cfg(test)]
include!("plan_resource_aggregate_tests.rs");
include!("plan_resource_details.rs");
