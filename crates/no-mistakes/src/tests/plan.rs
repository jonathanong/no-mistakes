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

    if let Some(framework) = args.framework {
        let forced_fallback = global_config_trigger(&root, &changed_files);
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

    // 2a. Analyze lockfile changes targeted
    let lockfile_analysis = super::lockfile_changes::analyze_lockfile_changes(
        args,
        &root,
        &collected.files,
    );
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
        "package.json"
            | "tsconfig.json"
            | ".no-mistakes.yml"
            | ".no-mistakes.yaml"
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

pub(crate) fn slash_node_name(node: &NodeId, root: &Path) -> String {
    match node {
        NodeId::File(p) => no_mistakes::codebase::ts_source::relative_slash_path(root, p),
        NodeId::Symbol { file, symbol } => {
            let rel = no_mistakes::codebase::ts_source::relative_slash_path(root, file);
            format!("{}#{}", rel, symbol)
        }
        NodeId::Module(specifier) => specifier.clone(),
        NodeId::QueueJob { queue_file, job } => {
            let rel = no_mistakes::codebase::ts_source::relative_slash_path(root, queue_file);
            format!("{}#{}", rel, job)
        }
    }
}

pub(crate) fn relative_path(root: &Path, absolute: &Path) -> String {
    no_mistakes::codebase::ts_source::relative_slash_path(root, absolute)
}

fn changed_start_nodes(graph: &DepGraph, changed: &Path, include_symbols: bool) -> Vec<NodeId> {
    symbol_aware_start_nodes(graph, changed, None, include_symbols)
}

pub(crate) fn symbol_aware_start_nodes(
    graph: &DepGraph,
    file: &Path,
    symbol: Option<&String>,
    include_symbols: bool,
) -> Vec<NodeId> {
    if let Some(symbol) = symbol.filter(|_| include_symbols) {
        return vec![NodeId::Symbol {
            file: file.to_path_buf(),
            symbol: symbol.clone(),
        }];
    }
    let file_node = NodeId::File(file.to_path_buf());
    let mut starts = vec![file_node.clone()];
    if include_symbols {
        if let Some(neighbors) = graph.dependencies_of_node(&file_node) {
            starts.extend(neighbors.iter().filter_map(|(node, _)| match node {
                NodeId::Symbol {
                    file: symbol_file, ..
                } if symbol_file == file => Some(node.clone()),
                _ => None,
            }));
        }
    }
    starts.sort();
    starts.dedup();
    starts
}

/// Custom BFS path finder in the reverse (dependents) direction.
/// Returns reachable test nodes, and a map of node -> (parent, edge_kind) for shortest paths.
#[allow(clippy::type_complexity)]
pub(crate) fn bfs_path_find(
    graph: &DepGraph,
    start: &NodeId,
    test_filter: &TestFileFilter,
    root: &Path,
) -> (
    Vec<(NodeId, Vec<EdgeKind>)>,
    HashMap<NodeId, (NodeId, EdgeKind)>,
) {
    let mut queue = VecDeque::new();
    let mut parents: HashMap<NodeId, (NodeId, EdgeKind)> = HashMap::new();
    let mut visited = HashSet::new();
    let mut owner_widened_files = HashSet::new();
    let mut reachable = Vec::new();

    queue.push_back(start.clone());
    visited.insert(start.clone());

    while let Some(current) = queue.pop_front() {
        // Check if current is a test file
        if let NodeId::File(p) = &current {
            if current != *start && test_filter.is_match(root, p) {
                // Reconstruct the path of edges to current
                let mut edge_path = Vec::new();
                let mut curr_node = current.clone();
                while let Some((parent, kind)) = parents.get(&curr_node) {
                    edge_path.push(*kind);
                    curr_node = parent.clone();
                }
                edge_path.reverse();
                reachable.push((current.clone(), edge_path));
            }
        }

        // Get dependents
        if let Some(neighbors) = graph.dependents_of_node(&current) {
            for (neighbor, kind) in neighbors {
                if owner_widened_files.contains(&current)
                    && !owner_widened_neighbor_allowed(
                        root,
                        test_filter,
                        graph,
                        neighbor,
                        neighbors,
                    )
                {
                    continue;
                }
                if let (NodeId::Symbol { file, .. }, NodeId::File(neighbor_file)) =
                    (&current, neighbor)
                {
                    if current == *start
                        && file == neighbor_file
                        && !test_filter.is_match(root, neighbor_file)
                    {
                        continue;
                    }
                }
                if !visited.contains(neighbor) {
                    if let (NodeId::Symbol { file, .. }, NodeId::File(neighbor_file)) =
                        (&current, neighbor)
                    {
                        if file == neighbor_file {
                            owner_widened_files.insert(neighbor.clone());
                        }
                    }
                    visited.insert(neighbor.clone());
                    parents.insert(neighbor.clone(), (current.clone(), *kind));
                    queue.push_back(neighbor.clone());
                }
            }
        }
    }

    (reachable, parents)
}

pub(crate) fn path_confidence(edges: &[EdgeKind]) -> Confidence {
    let mut conf = Confidence::High;
    for edge in edges {
        match edge {
            EdgeKind::HttpCall
            | EdgeKind::ProcessSpawn
            | EdgeKind::QueueEnqueue
            | EdgeKind::QueueWorker
            | EdgeKind::RouteRef
            | EdgeKind::Layout
            | EdgeKind::RouteTest
            | EdgeKind::Selector
            | EdgeKind::AssetImport
            | EdgeKind::ReactRender
            | EdgeKind::PackageDependency => return Confidence::Low,
            EdgeKind::DynamicImport => conf = Confidence::Medium,
            _ => {}
        }
    }
    conf
}

pub(crate) fn impact_reason_label(edge: EdgeKind) -> &'static str {
    match edge {
        EdgeKind::Import
        | EdgeKind::TypeImport
        | EdgeKind::DynamicImport
        | EdgeKind::Require
        | EdgeKind::WorkspaceImport => "dependency",
        EdgeKind::PackageDependency => "package-json dependency",
        EdgeKind::RouteRef | EdgeKind::RouteTest => "route",
        EdgeKind::Layout => "layout",
        EdgeKind::TestOf => "test",
        EdgeKind::QueueEnqueue | EdgeKind::QueueWorker => "queue",
        EdgeKind::MarkdownLink => "md",
        EdgeKind::CiInvocation => "ci",
        EdgeKind::HttpCall => "http",
        EdgeKind::ProcessSpawn => "process",
        EdgeKind::AssetImport => "asset",
        EdgeKind::ReactRender => "react-render",
        EdgeKind::Selector => "selector",
    }
}
