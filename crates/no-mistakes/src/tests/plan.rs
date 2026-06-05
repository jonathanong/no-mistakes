use crate::tests::comment::render_markdown_plan;
use crate::tests::{
    Confidence, ImpactReason, PlanArgs, PlanFormat, SelectedTest, TestPlan, Warning,
};
use anyhow::{Context, Result};
use no_mistakes::codebase::dependencies::graph::{DepGraph, EdgeKind, NodeId};
use no_mistakes::codebase::test_filter::TestFileFilter;
use no_mistakes::config::v2::load_v2_config;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

include!("plan_extra_inputs.rs");

pub(crate) fn run(args: PlanArgs) -> Result<ExitCode> {
    let plan = generate_plan(&args)?;

    let format = if args.json {
        PlanFormat::Json
    } else {
        args.format.unwrap_or(PlanFormat::Json)
    };

    match format {
        PlanFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&plan)?);
        }
        PlanFormat::Paths => {
            for test in &plan.selected_tests {
                println!("{}", test.test_file);
            }
        }
        PlanFormat::Markdown | PlanFormat::Md => {
            println!("{}", render_markdown_plan(&plan));
        }
    }

    Ok(ExitCode::SUCCESS)
}

const _: fn(PlanArgs) -> Result<ExitCode> = run;

pub fn generate_plan(args: &PlanArgs) -> Result<TestPlan> {
    let cwd = std::env::current_dir().context("cwd must be accessible")?;
    let root = no_mistakes::cli::resolve_optional_root(Some(&args.root), &cwd);
    let root = no_mistakes::codebase::ts_resolver::normalize_path(&root);
    let root = root.canonicalize().unwrap_or(root);

    let config = load_v2_config(&root, args.config.as_deref())?;
    let tsconfig = crate::tests::why::resolve_tsconfig(args.tsconfig.as_deref(), &root)?;

    // 1. Collect changed files
    let collected = super::changed_files::collect_changed_files(args, &root)?;
    let changed_files = super::changed_files::existing_changed_files(&collected);
    let deleted_files = &collected.deleted;

    // 2a. Analyze lockfile changes targeted
    let lockfile_analysis =
        super::lockfile_changes::analyze_lockfile_changes(args, &root, &collected.files);

    if let Some(framework) = args.framework {
        let forced_fallback = global_config_trigger(&root, &changed_files).or_else(|| {
            if lockfile_analysis.fallback_triggered {
                lockfile_analysis
                    .warnings
                    .first()
                    .map(|w| (w.message.clone(), root.join(&w.file)))
            } else {
                None
            }
        });
        return super::configured_plan::generate_configured_plan(
            args,
            framework,
            &root,
            &config,
            &tsconfig,
            &changed_files,
            &collected.diff_files,
            forced_fallback,
        );
    }
    let lockfile_changed_packages: Vec<(String, String)> = lockfile_analysis
        .diff_by_lockfile
        .iter()
        .flat_map(|(lockfile_path, lf_diff)| {
            let rel = relative_path(&root, lockfile_path);
            lf_diff
                .all_changed_names()
                .map(|name| (name.to_string(), rel.clone()))
                .collect::<Vec<_>>()
        })
        .collect();

    // 2b. Check for global configuration files (binary lockfiles or other global triggers)
    let has_binary_lockfile_fallback =
        global_config_fallback(args) && lockfile_analysis.fallback_triggered;

    if global_config_fallback(args) {
        let global_trigger = global_config_trigger(&root, &changed_files);
        let fallback_reason = if has_binary_lockfile_fallback && global_trigger.is_none() {
            lockfile_analysis
                .warnings
                .first()
                .map(|w| (w.message.clone(), root.join(&w.file)))
        } else {
            global_trigger
        };
        if let Some((reason, trigger_file)) = fallback_reason {
            let relative_changed = relative_path(&root, &trigger_file);
            let all_test_files = discover_all_tests(&root, &config)?;
            let mut selected_tests = Vec::new();
            for test in all_test_files {
                let rel_test = relative_path(&root, &test);
                selected_tests.push(SelectedTest {
                    test_file: rel_test.clone(),
                    confidence: Confidence::High,
                    targets: Vec::new(),
                    reasons: vec![ImpactReason {
                        changed_file: relative_changed.clone(),
                        path: vec![relative_changed.clone(), rel_test],
                        via: vec!["global configuration".to_string()],
                    }],
                });
            }
            selected_tests.sort_by(|a, b| a.test_file.cmp(&b.test_file));
            return Ok(TestPlan {
                selected_tests,
                groups: Vec::new(),
                warnings: Vec::new(),
                fallback_triggered: true,
                fallback_reason: Some(reason),
            });
        }
    }

    // 3. Build graph and test filter
    let graph = if args.include_symbols {
        no_mistakes::codebase::dependencies::graph::DepGraph::build_with_plan(
            root.as_path(),
            &tsconfig,
            no_mistakes::codebase::dependencies::graph::GraphBuildPlan::all().with_symbols(true),
        )?
    } else {
        DepGraph::build(root.as_path(), &tsconfig)?
    };
    let test_filter = TestFileFilter::new(root.as_path(), &config);

    let mut selected_map: HashMap<PathBuf, SelectedTest> = HashMap::new();
    let mut warnings = Vec::new();
    let mut warnings_seen = HashSet::new();

    // 4. Trace each changed file
    for changed in &changed_files {
        let basename = changed.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if no_mistakes::codebase::lockfile::detect_manager(basename).is_some()
            || no_mistakes::codebase::lockfile::is_binary_lockfile(basename)
        {
            continue;
        }

        let rel_changed = relative_path(&root, changed);

        // If the changed file is a test file itself, select it directly
        if test_filter.is_match(&root, changed) {
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
            };
            if !entry.reasons.contains(&reason) {
                entry.reasons.push(reason);
            }
            continue;
        }

        // Otherwise, run BFS path finder in reverse direction
        let start_nodes = changed_start_nodes(&graph, changed, args.include_symbols);

        for start_node in start_nodes {
            let (reachable_tests, path_parents) =
                bfs_path_find(&graph, &start_node, &test_filter, &root);

            for (test_node, edge_path) in reachable_tests {
                let test_path = match &test_node {
                    NodeId::File(p) => p.clone(),
                    _ => continue,
                };
                let rel_test = relative_path(&root, &test_path);

                // Compute confidence of the path
                let path_conf = path_confidence(&edge_path);

                // Reconstruct path node chain and collect warnings in a single pass
                let mut node_chain = Vec::new();
                let mut curr = test_node.clone();
                node_chain.push(slash_node_name(&curr, &root));

                while let Some((parent, kind)) = path_parents.get(&curr) {
                    node_chain.push(slash_node_name(parent, &root));

                    match kind {
                        EdgeKind::DynamicImport => {
                            let warn = Warning {
                                r#type: "dynamic-import".to_string(),
                                message: format!(
                                    "Dynamic import in `{}` might not be fully resolved.",
                                    slash_node_name(&curr, &root)
                                ),
                                file: slash_node_name(&curr, &root),
                            };
                            if warnings_seen.insert((warn.r#type.clone(), warn.file.clone())) {
                                warnings.push(warn);
                            }
                        }
                        EdgeKind::HttpCall => {
                            let warn = Warning {
                                r#type: "http-call".to_string(),
                                message: format!(
                                    "Dynamic HTTP call in `{}` to backend `{}`.",
                                    slash_node_name(&curr, &root),
                                    slash_node_name(parent, &root)
                                ),
                                file: slash_node_name(&curr, &root),
                            };
                            if warnings_seen.insert((warn.r#type.clone(), warn.file.clone())) {
                                warnings.push(warn);
                            }
                        }
                        EdgeKind::ProcessSpawn => {
                            let warn = Warning {
                                r#type: "process-spawn".to_string(),
                                message: format!(
                                    "Process spawned in `{}`.",
                                    slash_node_name(&curr, &root)
                                ),
                                file: slash_node_name(&curr, &root),
                            };
                            if warnings_seen.insert((warn.r#type.clone(), warn.file.clone())) {
                                warnings.push(warn);
                            }
                        }
                        _ => {}
                    }
                    curr = parent.clone();
                }
                node_chain.reverse();

                let via_strings: Vec<String> = edge_path
                    .iter()
                    .map(|k| impact_reason_label(*k).to_string())
                    .collect();

                let reason = ImpactReason {
                    changed_file: rel_changed.clone(),
                    path: node_chain,
                    via: via_strings,
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

    // 4b. Trace lockfile package dependency changes
    for (pkg_name, lockfile_rel) in &lockfile_changed_packages {
        let start_node = NodeId::Module(pkg_name.clone());
        let (reachable_tests, path_parents) =
            bfs_path_find(&graph, &start_node, &test_filter, &root);

        for (test_node, edge_path) in reachable_tests {
            let test_path = match &test_node {
                NodeId::File(p) => p.clone(),
                _ => continue,
            };
            let rel_test = relative_path(&root, &test_path);
            let path_conf = path_confidence(&edge_path);

            let mut node_chain = Vec::new();
            let mut curr = test_node.clone();
            node_chain.push(slash_node_name(&curr, &root));
            while let Some((parent, _)) = path_parents.get(&curr) {
                node_chain.push(slash_node_name(parent, &root));
                curr = parent.clone();
            }
            node_chain.reverse();

            let via_strings: Vec<String> = edge_path
                .iter()
                .map(|k| impact_reason_label(*k).to_string())
                .collect();

            let reason = ImpactReason {
                changed_file: lockfile_rel.clone(),
                path: node_chain,
                via: via_strings,
            };

            let entry = selected_map
                .entry(test_path)
                .or_insert_with(|| SelectedTest {
                    test_file: rel_test.clone(),
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

    // Merge lockfile analysis warnings
    for warn in lockfile_analysis.warnings {
        if warnings_seen.insert((warn.r#type.clone(), warn.file.clone())) {
            warnings.push(warn);
        }
    }

    // 5. Trace deleted files (phantom node lookup in reverse map)
    trace_deleted_files(
        deleted_files,
        &graph,
        &test_filter,
        &root,
        &mut selected_map,
        &mut warnings,
        &mut warnings_seen,
    );

    // 6. Trace entrypoints (file#export)
    trace_entrypoints(
        &args.entrypoints,
        &args.entrypoint_symbols,
        &graph,
        &test_filter,
        &root,
        &mut selected_map,
        args.include_symbols,
    )?;

    let mut selected_tests: Vec<SelectedTest> = selected_map.into_values().collect();
    for test in &mut selected_tests {
        test.reasons
            .sort_by(|a, b| a.changed_file.cmp(&b.changed_file));
    }
    selected_tests.sort_by(|a, b| a.test_file.cmp(&b.test_file));
    warnings.sort_by(|a, b| (&a.file, &a.message).cmp(&(&b.file, &b.message)));

    Ok(TestPlan {
        selected_tests,
        groups: Vec::new(),
        warnings,
        fallback_triggered: false,
        fallback_reason: None,
    })
}

fn global_config_fallback(args: &PlanArgs) -> bool {
    args.global_config_fallback.unwrap_or(false)
}

pub(crate) fn global_config_trigger(
    root: &Path,
    changed_files: &[PathBuf],
) -> Option<(String, PathBuf)> {
    changed_files.iter().find_map(|file| {
        let relative_changed = relative_path(root, file);
        is_global_config_path(root, file, &relative_changed).then(|| {
            (
                format!("Global configuration file changed: {}", relative_changed),
                file.clone(),
            )
        })
    })
}

fn is_global_config_path(root: &Path, absolute: &Path, relative: &str) -> bool {
    if matches!(
        relative,
        "package.json" | "tsconfig.json" | ".no-mistakes.yml" | ".no-mistakes.yaml"
    ) {
        return true;
    }

    let Some(name) = absolute.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if !matches!(
        name,
        "next.config.js"
            | "next.config.mjs"
            | "next.config.ts"
            | "next.config.mts"
            | "proxy.js"
            | "proxy.mjs"
            | "proxy.ts"
            | "proxy.mts"
            | "middleware.js"
            | "middleware.mjs"
            | "middleware.ts"
            | "middleware.mts"
    ) {
        return false;
    }

    let Some(parent) = absolute.parent() else {
        return false;
    };
    parent == root || next_project_root(parent)
}

fn next_project_root(path: &Path) -> bool {
    path.join("app").is_dir()
        || path.join("pages").is_dir()
        || path.join("src/app").is_dir()
        || path.join("src/pages").is_dir()
}

fn discover_all_tests(
    root: &Path,
    config: &no_mistakes::config::v2::NoMistakesConfig,
) -> Result<Vec<PathBuf>> {
    let filter = TestFileFilter::new(root, config);
    Ok(
        no_mistakes::codebase::ts_source::discover_files(root, &config.filesystem.skip_directories)
            .into_iter()
            .filter(|f| filter.is_match(root, f))
            .collect(),
    )
}

include!("plan_bfs.rs");
