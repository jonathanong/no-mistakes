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
mod global_config;
mod run;
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
    let config = &prepared.config;
    let collected = &prepared.collected;
    let changed_files = prepared.planning_changed_files(args.framework);
    let deleted_files = &collected.deleted;
    let lockfile_analysis = &prepared.lockfile_analysis;

    if let Some(framework) = args.framework {
        if args.direct_test_owner {
            return super::configured_plan::generate_direct_test_owner_plan_with_prepared(
                framework, prepared,
            );
        }
        // Compute lockfile changed packages for BFS tracing in framework plans — same
        // structure as the non-framework §4b path below. Parseable lockfile diffs no
        // longer force an unconditional full-suite fallback; we wire the packages into
        // the configured-plan dependencies group instead.
        // fallback_triggered means binary / invalid-ref / diff-only — no parseable diff available.
        // Full-suite selection still requires the effective global fallback opt-in.
        let javascript_dependencies =
            super::prepared_plan::javascript_dependency_framework(Some(framework));
        let forced_fallback = prepared
            .framework_config_trigger(framework)
            .or_else(|| {
                global_config::excluding_v2_config(
                    root,
                    &collected.files,
                    Some(framework),
                    prepared,
                )
            })
            .or_else(|| {
                if javascript_dependencies && lockfile_analysis.fallback_triggered {
                    lockfile_analysis
                        .warnings
                        .first()
                        .map(|w| (w.message.clone(), root.join(&w.file)))
                } else {
                    None
                }
            })
            .or_else(|| {
                (javascript_dependencies && prepared.package_manifest_analysis.fallback_triggered)
                    .then(|| {
                        let warning = prepared
                            .package_manifest_analysis
                            .warnings
                            .first()
                            .expect("package manifest fallback warning");
                        (warning.message.clone(), root.join(&warning.file))
                    })
            })
            .or_else(|| {
                if framework == super::TestFramework::Swift
                    && prepared.swift_resolved_analysis.fallback_triggered
                {
                    prepared
                        .swift_resolved_analysis
                        .warnings
                        .first()
                        .map(|warning| (warning.message.clone(), root.join(&warning.file)))
                } else {
                    None
                }
            })
            .or_else(|| {
                (framework == super::TestFramework::Swift
                    && prepared.swift_manifest_analysis.fallback_triggered)
                    .then(|| {
                        let warning = prepared
                            .swift_manifest_analysis
                            .warnings
                            .first()
                            .expect("fallback warning");
                        (warning.message.clone(), root.join(&warning.file))
                    })
            })
            .or_else(|| {
                (framework == super::TestFramework::Dotnet
                    && prepared.dotnet_dependency_analysis.fallback_triggered)
                    .then(|| {
                        let warning = prepared
                            .dotnet_dependency_analysis
                            .warnings
                            .first()
                            .expect(".NET dependency fallback warning");
                        (warning.message.clone(), root.join(&warning.file))
                    })
            });
        let discovered_tests = super::configured_plan::discover_framework_tests_from_prepared(
            args, framework, prepared,
        )?;
        let mut plan = super::configured_plan::generate_configured_plan_with_prepared(
            args,
            framework,
            root,
            config,
            &changed_files,
            deleted_files,
            &collected.diff_files,
            if javascript_dependencies {
                &prepared.lockfile_changed_packages
            } else {
                &[]
            },
            &prepared.workspace_map,
            forced_fallback,
            discovered_tests,
            prepared,
            timing,
        )?;
        let mut warning_keys: HashSet<WarningKey> = plan.warnings.iter().map(warning_key).collect();
        let dependency_warnings = javascript_dependencies
            .then_some(lockfile_analysis.warnings.iter())
            .into_iter()
            .flatten()
            .chain(
                javascript_dependencies
                    .then_some(prepared.package_manifest_analysis.warnings.iter())
                    .into_iter()
                    .flatten(),
            )
            .chain(
                (framework == super::TestFramework::Swift)
                    .then_some(prepared.swift_resolved_analysis.warnings.iter())
                    .into_iter()
                    .flatten(),
            )
            .chain(
                (framework == super::TestFramework::Swift)
                    .then_some(prepared.swift_manifest_analysis.warnings.iter())
                    .into_iter()
                    .flatten(),
            )
            .chain(
                (framework == super::TestFramework::Dotnet)
                    .then_some(prepared.dotnet_dependency_analysis.warnings.iter())
                    .into_iter()
                    .flatten(),
            );
        for warning in dependency_warnings {
            if warning_keys.insert(warning_key(warning)) {
                plan.warnings.push(warning.clone());
            }
        }
        return Ok(plan);
    }

    // 2b. Determine fallback trigger.
    //
    // Every full-suite fallback is explicit opt-in, including diff-only and
    // binary lockfiles whose contents cannot be analyzed.
    let fallback_reason = if global_config_fallback(args)
        && (lockfile_analysis.fallback_triggered
            || prepared.package_manifest_analysis.fallback_triggered
            || prepared.swift_resolved_analysis.fallback_triggered
            || prepared.swift_manifest_analysis.fallback_triggered
            || prepared.dotnet_dependency_analysis.fallback_triggered)
    {
        lockfile_analysis
            .warnings
            .first()
            .or_else(|| prepared.package_manifest_analysis.warnings.first())
            .or_else(|| prepared.swift_resolved_analysis.warnings.first())
            .or_else(|| prepared.swift_manifest_analysis.warnings.first())
            .or_else(|| prepared.dotnet_dependency_analysis.warnings.first())
            .map(|warning| (warning.message.clone(), root.join(&warning.file)))
    } else if global_config_fallback(args) {
        global_config_trigger(root, &changed_files, None, prepared)
    } else {
        None
    };

    if let Some((reason, trigger_file)) = fallback_reason {
        let relative_changed = relative_path(root, &trigger_file);
        let all_test_files = global_config::discover_all_tests_from_prepared(prepared);
        let mut selected_tests = Vec::new();
        for test in all_test_files {
            let rel_test = relative_path(root, &test);
            selected_tests.push(SelectedTest {
                test_file: rel_test.clone(),
                confidence: Confidence::High,
                targets: Vec::new(),
                reasons: vec![ImpactReason {
                    changed_file: relative_changed.clone(),
                    path: vec![relative_changed.clone(), rel_test],
                    via: vec!["global configuration".to_string()],
                    via_details: Vec::new(),
                }],
            });
        }
        selected_tests.sort_by(|a, b| a.test_file.cmp(&b.test_file));
        return Ok(TestPlan {
            changed_files: Vec::new(),
            selected_tests,
            groups: Vec::new(),
            warnings: {
                let mut warnings = lockfile_analysis.warnings.clone();
                warnings.extend(prepared.package_manifest_analysis.warnings.iter().cloned());
                warnings.extend(prepared.swift_resolved_analysis.warnings.iter().cloned());
                warnings.extend(prepared.swift_manifest_analysis.warnings.iter().cloned());
                warnings.extend(prepared.dotnet_dependency_analysis.warnings.iter().cloned());
                warnings.extend(prepared.tsconfig_warnings());
                warnings
            },
            fallback_triggered: true,
            fallback_reason: Some(reason),
            ..Default::default()
        });
    }

    // 3. Build graph and test filter
    let graph = prepared.graph()?;
    let test_filter = prepared.test_filter().clone();
    let all_test_files = global_config::discover_all_tests_from_prepared(prepared);
    let all_test_set = all_test_files.iter().cloned().collect();
    let native_semantic_seeds =
        crate::tests::configured_plan::native_semantic_seeds::native_semantic_seed_candidates(
            root,
            prepared,
            graph,
            &all_test_set,
            None,
        );

    let mut selected_map: HashMap<PathBuf, SelectedTest> = HashMap::new();
    let mut warnings = prepared.tsconfig_warnings();
    let mut warnings_seen: HashSet<WarningKey> = warnings.iter().map(warning_key).collect();

    // 4. Trace each changed file
    for changed in &changed_files {
        let basename = changed.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if no_mistakes::codebase::lockfile::detect_manager(basename).is_some()
            || no_mistakes::codebase::lockfile::is_binary_lockfile(basename)
        {
            continue;
        }

        let rel_changed = relative_path(root, changed);

        // Dynamic resource calls are intentionally edge-less. A changed
        // consumer remains relevant even when no static path reaches a test.
        push_resource_diagnostics(graph, root, changed, &mut warnings, &mut warnings_seen);

        // If the changed file is a test file itself, select it directly
        if test_filter.is_match(root, changed) {
            let entry = selected_map
                .entry(changed.clone())
                .or_insert_with(|| SelectedTest {
                    test_file: rel_changed.clone(),
                    confidence: Confidence::High,
                    targets: Vec::new(),
                    reasons: Vec::new(),
                });
            entry.confidence = Confidence::High;
            let reason = ImpactReason {
                changed_file: rel_changed.clone(),
                path: vec![rel_changed.clone()],
                via: vec!["self".to_string()],
                via_details: Vec::new(),
            };
            if !entry.reasons.contains(&reason) {
                entry.reasons.push(reason);
            }
            continue;
        }

        // Otherwise, run BFS path finder in reverse direction
        let start_nodes = changed_start_nodes(graph, changed, args.include_symbols);

        for start_node in start_nodes {
            let (reachable_tests, path_parents) =
                bfs_path_find(graph, &start_node, &test_filter, root);

            for (test_node, edge_path) in reachable_tests {
                let test_path = match &test_node {
                    NodeId::File(p) => p.to_path_buf(),
                    _ => continue,
                };
                let rel_test = relative_path(root, &test_path);

                // Compute confidence of the path
                let path_conf = path_confidence(&edge_path);

                // Reconstruct path node chain and collect warnings in a single pass
                let mut node_chain = Vec::new();
                let mut reverse_details = Vec::new();
                let mut curr = test_node.clone();
                node_chain.push(slash_node_name(&curr, root));

                while let Some((parent, kind)) = path_parents.get(&curr) {
                    if let Some(file) = curr.as_file() {
                        push_resource_diagnostics(
                            graph,
                            root,
                            file,
                            &mut warnings,
                            &mut warnings_seen,
                        );
                    }
                    node_chain.push(slash_node_name(parent, root));
                    reverse_details.push(resource_edge_detail(graph, &curr, parent, *kind, root));

                    match kind {
                        EdgeKind::DynamicImport => {
                            let warn = Warning {
                                r#type: "dynamic-import".to_string(),
                                message: format!(
                                    "Dynamic import in `{}` might not be fully resolved.",
                                    slash_node_name(&curr, root)
                                ),
                                file: slash_node_name(&curr, root),
                                line: None,
                            };
                            if warnings_seen.insert(warning_key(&warn)) {
                                warnings.push(warn);
                            }
                        }
                        EdgeKind::HttpCall => {
                            let warn = Warning {
                                r#type: "http-call".to_string(),
                                message: format!(
                                    "Dynamic HTTP call in `{}` to backend `{}`.",
                                    slash_node_name(&curr, root),
                                    slash_node_name(parent, root)
                                ),
                                file: slash_node_name(&curr, root),
                                line: None,
                            };
                            if warnings_seen.insert(warning_key(&warn)) {
                                warnings.push(warn);
                            }
                        }
                        EdgeKind::ProcessSpawn => {
                            let warn = Warning {
                                r#type: "process-spawn".to_string(),
                                message: format!(
                                    "Process spawned in `{}`.",
                                    slash_node_name(&curr, root)
                                ),
                                file: slash_node_name(&curr, root),
                                line: None,
                            };
                            if warnings_seen.insert(warning_key(&warn)) {
                                warnings.push(warn);
                            }
                        }
                        _ => {}
                    }
                    curr = parent.clone();
                }
                if let Some(file) = curr.as_file() {
                    push_resource_diagnostics(graph, root, file, &mut warnings, &mut warnings_seen);
                }
                node_chain.reverse();
                reverse_details.reverse();

                let via_strings: Vec<String> = edge_path
                    .iter()
                    .map(|k| impact_reason_label(*k).to_string())
                    .collect();

                let reason = ImpactReason {
                    changed_file: rel_changed.clone(),
                    path: node_chain,
                    via: via_strings,
                    via_details: reverse_details,
                };

                let entry = selected_map
                    .entry(test_path)
                    .or_insert_with(|| SelectedTest {
                        test_file: rel_test.clone(),
                        confidence: path_conf,
                        targets: Vec::new(),
                        reasons: Vec::new(),
                    });

                // Update confidence to the highest among paths
                if path_conf > entry.confidence {
                    entry.confidence = path_conf;
                }

                if !entry.reasons.contains(&reason) {
                    entry.reasons.push(reason);
                }
            }
        }
    }

    if let Some(fallback_plan) = dependency_seeds::trace_and_fallback(
        args,
        prepared,
        graph,
        &test_filter,
        &all_test_files,
        &native_semantic_seeds,
        &mut selected_map,
        &mut warnings,
        &mut warnings_seen,
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
