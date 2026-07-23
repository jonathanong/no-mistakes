use crate::tests::plan::{
    bfs_path_find, path_confidence, relative_path, resource_edge_detail, slash_node_name,
    symbol_aware_start_nodes,
};
use crate::tests::{
    push_resource_diagnostics, warning_key, Confidence, ImpactArgs, ImpactReason, PlanFormat,
    SelectedTest, TestPlan, Warning, WarningKey,
};
use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use no_mistakes::codebase::dependencies::graph::{DepGraph, EdgeKind, NodeId};
use no_mistakes::codebase::dependencies::parse_entrypoint;
use no_mistakes::config::v2::load_v2_config;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

pub(crate) fn run(args: ImpactArgs) -> Result<ExitCode> {
    let plan = generate_impact_plan(&args)?;

    let format = if args.json {
        PlanFormat::Json
    } else {
        args.format.unwrap_or(PlanFormat::Json)
    };
    let output = super::plan_output::render(&plan, format, "tests impact")?;
    crate::invocation::commit_timeout()?;
    print!("{output}");

    Ok(ExitCode::SUCCESS)
}

const _: fn(ImpactArgs) -> Result<ExitCode> = run;

pub fn generate_impact_plan(args: &ImpactArgs) -> Result<TestPlan> {
    let cwd = std::env::current_dir().context("cwd must be accessible")?;
    let root = no_mistakes::cli::resolve_optional_root(Some(&args.root), &cwd);
    let root = no_mistakes::codebase::ts_resolver::normalize_path(&root);
    let root = root.canonicalize().unwrap_or(root);

    let config = load_v2_config(&root, args.config.as_deref())?;
    let impact_graph = crate::tests::build_test_impact_graph(
        root.as_path(),
        args.tsconfig.as_deref(),
        &config,
        args.config.as_deref(),
        args.include_symbols,
    )?;
    let graph = &impact_graph.graph;
    let test_filter = &impact_graph.test_filter;
    let registry_set = compile_registry_globset(&config.tests.impact.registries);

    let mut selected_map: HashMap<PathBuf, SelectedTest> = HashMap::new();
    let mut warnings = Vec::new();
    let mut warnings_seen: HashSet<WarningKey> = HashSet::new();
    let mut registry_seen: HashSet<(String, String)> = HashSet::new();
    let mut changed_files = Vec::new();
    let mut deleted_files = Vec::new();

    for (index, raw) in args.entrypoints.iter().enumerate() {
        let structured_symbol = args.entrypoint_symbols.get(index).cloned().flatten();
        let structured_entrypoint = structured_symbol.is_some();
        let (raw_file, parsed_symbol) = if structured_entrypoint {
            (PathBuf::from(raw), None)
        } else {
            parse_entrypoint(raw)
        };
        let symbol = structured_symbol
            .filter(|symbol| !symbol.is_empty())
            .or(parsed_symbol);
        if symbol.is_some() && !args.include_symbols {
            anyhow::bail!(
                "Entrypoint `{}` uses `#symbol`; pass --symbols to enable symbol traversal.",
                raw
            );
        }
        let file = if raw_file.is_absolute() {
            raw_file
        } else {
            root.join(&raw_file)
        };
        let normalized = no_mistakes::codebase::ts_resolver::normalize_path(&file);
        if impact_graph.visible_files.contains(&normalized) {
            changed_files.push(normalized.clone());
        } else {
            deleted_files.push(normalized.clone());
        }
        let start_nodes =
            symbol_aware_start_nodes(graph, &normalized, symbol.as_ref(), args.include_symbols);
        let rel_changed = symbol
            .as_ref()
            .filter(|_| args.include_symbols)
            .map_or_else(
                || relative_path(&root, &normalized),
                |symbol| format!("{}#{}", relative_path(&root, &normalized), symbol),
            );

        // A dynamic resource call has no edge to traverse, but a directly
        // changed consumer is still relevant to this impact query.
        push_resource_diagnostics(graph, &root, &normalized, &mut warnings, &mut warnings_seen);

        // Registry hints are file-level ("this file is registered in X"); a
        // symbol-scoped entrypoint asks about one export, so a file-level hint
        // could be unrelated. Only emit for whole-file entrypoints.
        if symbol.is_none() {
            if let Some(registry_set) = registry_set.as_ref() {
                push_registry_hints(
                    graph,
                    &normalized,
                    &root,
                    registry_set,
                    &mut warnings,
                    &mut registry_seen,
                );
            }
        }

        if test_filter.is_match(&root, &normalized) {
            let rel_test = relative_path(&root, &normalized);
            let entry = selected_map
                .entry(normalized.clone())
                .or_insert_with(|| SelectedTest {
                    test_file: rel_test,
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

        for start_node in start_nodes {
            let (reachable_tests, path_parents) =
                bfs_path_find(graph, &start_node, test_filter, &root);

            for (test_node, edge_path) in reachable_tests {
                let test_path = match &test_node {
                    NodeId::File(p) => p.clone(),
                    _ => continue,
                };
                let rel_test = relative_path(&root, &test_path);
                let path_conf = path_confidence(&edge_path);

                let mut node_chain = Vec::new();
                let mut reverse_details = Vec::new();
                let mut curr = test_node.clone();
                node_chain.push(slash_node_name(&curr, &root));

                while let Some((parent, kind)) = path_parents.get(&curr) {
                    if let Some(file) = curr.as_file() {
                        push_resource_diagnostics(
                            graph,
                            &root,
                            file,
                            &mut warnings,
                            &mut warnings_seen,
                        );
                    }
                    node_chain.push(slash_node_name(parent, &root));
                    reverse_details.push(resource_edge_detail(graph, &curr, parent, *kind, &root));
                    push_warning(
                        &root,
                        &curr,
                        parent,
                        *kind,
                        &mut warnings,
                        &mut warnings_seen,
                    );
                    curr = parent.clone();
                }
                if let Some(file) = curr.as_file() {
                    push_resource_diagnostics(
                        graph,
                        &root,
                        file,
                        &mut warnings,
                        &mut warnings_seen,
                    );
                }
                node_chain.reverse();
                reverse_details.reverse();

                let via_strings: Vec<String> = edge_path
                    .iter()
                    .map(|k| crate::tests::plan::impact_reason_label(*k).to_string())
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
                        test_file: rel_test,
                        confidence: path_conf,
                        targets: Vec::new(),
                        reasons: Vec::new(),
                    });

                if path_conf > entry.confidence {
                    entry.confidence = path_conf;
                }
                if !entry.reasons.contains(&reason) {
                    entry.reasons.push(reason);
                }
            }
        }
    }

    for warning in crate::tests::configured_plan::vitest_setup_fallback::warnings(
        &root,
        Some(&impact_graph.vitest_projects),
    ) {
        if warnings_seen.insert(warning_key(&warning)) {
            warnings.push(warning);
        }
    }
    let used = selected_map
        .values()
        .map(|test| test.test_file.clone())
        .collect::<HashSet<_>>();
    let vitest_fallback = crate::tests::configured_plan::vitest_setup_fallback::selection(
        &root,
        &changed_files,
        &deleted_files,
        Some(&impact_graph.vitest_projects),
        &impact_graph.vitest_discovered,
        &used,
        usize::MAX,
    );
    if let Some((_, picked)) = &vitest_fallback {
        for test in picked {
            selected_map
                .entry(root.join(&test.test_file))
                .and_modify(|existing| {
                    crate::tests::configured_plan_candidates::merge_selected(existing, test)
                })
                .or_insert_with(|| test.clone());
        }
    }

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
        selected_tests,
        groups: Vec::new(),
        warnings,
        fallback_triggered: vitest_fallback.is_some(),
        fallback_reason: vitest_fallback.map(|(reason, _)| reason),
    })
}

/// Compile the opt-in registry glob list. Returns `None` when unconfigured or
/// when every pattern is malformed. Malformed patterns are skipped so a single
/// bad glob does not silently disable registry hints for the valid ones, and the
/// impact query never fails on a bad glob.
fn compile_registry_globset(patterns: &[String]) -> Option<GlobSet> {
    if patterns.is_empty() {
        return None;
    }
    let mut builder = GlobSetBuilder::new();
    let mut has_valid = false;
    for pattern in patterns {
        if let Ok(glob) = Glob::new(pattern) {
            builder.add(glob);
            has_valid = true;
        }
    }
    has_valid.then(|| builder.build().ok()).flatten()
}

/// Emit a hint for each direct dependent of `target` whose file matches a
/// configured registry glob. Deduped per (target, registry) pair so each
/// changed file gets its own reminder for each registry it appears in.
fn push_registry_hints(
    graph: &DepGraph,
    target: &Path,
    root: &Path,
    registry_set: &GlobSet,
    warnings: &mut Vec<Warning>,
    registry_seen: &mut HashSet<(String, String)>,
) {
    let Some(dependents) = graph.dependents_of_node(&NodeId::File(target.to_path_buf())) else {
        return;
    };
    let target_rel = relative_path(root, target);
    for (dependent, kind) in dependents {
        // A type-only reference does not "register" a runtime entry, so it must
        // not produce a registry hint.
        if *kind == EdgeKind::TypeImport {
            continue;
        }
        if let NodeId::File(dep_path) = dependent {
            let registry_rel = relative_path(root, dep_path);
            if registry_set.is_match(&registry_rel)
                && registry_seen.insert((target_rel.clone(), registry_rel.clone()))
            {
                warnings.push(Warning {
                    r#type: "registry-hint".to_string(),
                    message: format!(
                        "`{}` is registered in `{}`; verify the registry entry is up to date",
                        target_rel, registry_rel
                    ),
                    file: registry_rel,
                    line: None,
                });
            }
        }
    }
}

fn push_warning(
    root: &Path,
    curr: &NodeId,
    parent: &NodeId,
    kind: EdgeKind,
    warnings: &mut Vec<Warning>,
    warnings_seen: &mut HashSet<WarningKey>,
) {
    let (warn_type, message, file) = match kind {
        EdgeKind::DynamicImport => {
            let file = slash_node_name(curr, root);
            (
                "dynamic-import",
                format!("Dynamic import in `{}` might not be fully resolved.", file),
                file,
            )
        }
        EdgeKind::HttpCall => {
            let file = slash_node_name(curr, root);
            (
                "http-call",
                format!(
                    "Dynamic HTTP call in `{}` to backend `{}`.",
                    file,
                    slash_node_name(parent, root)
                ),
                file,
            )
        }
        EdgeKind::ProcessSpawn => {
            let file = slash_node_name(curr, root);
            (
                "process-spawn",
                format!("Process spawned in `{}`.", file),
                file,
            )
        }
        _ => return,
    };
    let warn = Warning {
        r#type: warn_type.to_string(),
        message,
        file,
        line: None,
    };
    if warnings_seen.insert(warning_key(&warn)) {
        warnings.push(warn);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn impact_classifies_missing_setup_helper_entrypoints_as_deleted() {
        let root = no_mistakes::codebase::ts_resolver::normalize_path(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../fixtures/test-plan/vitest-setup-dependencies"),
        );
        let plan = generate_impact_plan(&ImpactArgs {
            entrypoints: vec!["runtime-owner/setup/deleted-runtime-helper.ts".to_string()],
            entrypoint_symbols: Vec::new(),
            include_symbols: false,
            root,
            config: None,
            tsconfig: None,
            format: None,
            json: true,
        })
        .unwrap();

        assert!(plan.fallback_triggered, "{plan:#?}");
        assert_eq!(
            plan.selected_tests[0].test_file,
            "runtime-owner/runtime-owner.test.ts"
        );
        assert!(plan.fallback_reason.as_deref().is_some_and(|reason| {
            reason.contains("transitive dependency of a resolved setup was deleted")
        }));
    }
}
