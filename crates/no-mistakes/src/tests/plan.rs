use crate::tests::comment::render_markdown_plan;
use crate::tests::{
    Confidence, ImpactReason, PlanArgs, PlanFormat, SelectedTest, TestPlan, Warning,
};
use anyhow::{Context, Result};
use no_mistakes_core::codebase::dependencies::graph::{DepGraph, EdgeKind, NodeId};
use no_mistakes_core::codebase::test_filter::TestFileFilter;
use no_mistakes_core::config::v2::load_v2_config;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

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

pub fn generate_plan(args: &PlanArgs) -> Result<TestPlan> {
    let cwd = std::env::current_dir().context("cwd must be accessible")?;
    let root = no_mistakes_core::cli::resolve_optional_root(Some(&args.root), &cwd);
    let root = no_mistakes_core::codebase::ts_resolver::normalize_path(&root);
    let root = root.canonicalize().unwrap_or(root);

    let config = load_v2_config(&root, args.config.as_deref())?;
    let tsconfig = crate::tests::why::resolve_tsconfig(args.tsconfig.as_deref(), &root)?;

    // 1. Collect changed files
    let changed_files = collect_changed_files(args, &root)?;

    // 2. Check for global configuration files
    for file in &changed_files {
        let relative_changed = relative_path(&root, file);
        if is_global_config_path(&relative_changed) {
            // Trigger fallback
            let all_test_files = discover_all_tests(&root, &config)?;
            let mut selected_tests = Vec::new();
            for test in all_test_files {
                let rel_test = relative_path(&root, &test);
                selected_tests.push(SelectedTest {
                    test_file: rel_test.clone(),
                    confidence: Confidence::High,
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
                warnings: Vec::new(),
                fallback_triggered: true,
                fallback_reason: Some(format!(
                    "Global configuration file changed: {}",
                    relative_changed
                )),
            });
        }
    }

    // 3. Build graph and test filter
    let graph = DepGraph::build(root.as_path(), &tsconfig)?;
    let test_filter = TestFileFilter::new(root.as_path(), &config);

    let mut selected_map: HashMap<PathBuf, SelectedTest> = HashMap::new();
    let mut warnings = Vec::new();
    let mut warnings_seen = HashSet::new();

    // 4. Trace each changed file
    for changed in &changed_files {
        let rel_changed = relative_path(&root, changed);

        // If it does not exist, add a warning
        if !changed.exists() {
            let warn = Warning {
                r#type: "file-not-found".to_string(),
                message: format!("Changed file not found on disk: {}", rel_changed),
                file: rel_changed.clone(),
            };
            if warnings_seen.insert((warn.r#type.clone(), warn.file.clone())) {
                warnings.push(warn);
            }
        }

        // If the changed file is a test file itself, select it directly
        if test_filter.is_match(&root, changed) {
            let entry = selected_map
                .entry(changed.clone())
                .or_insert_with(|| SelectedTest {
                    test_file: rel_changed.clone(),
                    confidence: Confidence::High,
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
        let start_node = NodeId::File(changed.clone());
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

    let mut selected_tests: Vec<SelectedTest> = selected_map.into_values().collect();
    for test in &mut selected_tests {
        test.reasons
            .sort_by(|a, b| a.changed_file.cmp(&b.changed_file));
    }
    selected_tests.sort_by(|a, b| a.test_file.cmp(&b.test_file));
    warnings.sort_by(|a, b| (&a.file, &a.message).cmp(&(&b.file, &b.message)));

    Ok(TestPlan {
        selected_tests,
        warnings,
        fallback_triggered: false,
        fallback_reason: None,
    })
}

fn collect_changed_files(args: &PlanArgs, root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    // From changed-file arguments
    for f in &args.changed_file {
        let path = if f.is_absolute() {
            f.clone()
        } else {
            root.join(f)
        };
        let resolved = path
            .canonicalize()
            .unwrap_or_else(|_| no_mistakes_core::codebase::ts_resolver::normalize_path(&path));
        files.push(resolved);
    }

    // From changed-files file list
    if let Some(ref path) = args.changed_files {
        let content = fs::read_to_string(path).with_context(|| {
            format!("Failed to read changed-files list from {}", path.display())
        })?;
        for line in content.lines() {
            let line = line.trim();
            if !line.is_empty() {
                let p = PathBuf::from(line);
                let path = if p.is_absolute() { p } else { root.join(p) };
                let resolved = path.canonicalize().unwrap_or_else(|_| {
                    no_mistakes_core::codebase::ts_resolver::normalize_path(&path)
                });
                files.push(resolved);
            }
        }
    }

    // From git if base or no inputs are provided
    if args.base.is_some() || (args.changed_file.is_empty() && args.changed_files.is_none()) {
        match get_git_changed_files(root, args.base.as_deref(), args.head.as_deref()) {
            Ok(git_files) => {
                for f in git_files {
                    files.push(root.join(f));
                }
            }
            Err(e) => {
                // If explicitly requested, fail
                if args.base.is_some() {
                    return Err(e);
                }
                // Otherwise, fail silently / log warning
                eprintln!("warning: failed to retrieve changed files from git: {}", e);
            }
        }
    }

    // Deduplicate and normalize
    let mut unique = HashSet::new();
    let mut result = Vec::new();
    for f in files {
        let normalized = no_mistakes_core::codebase::ts_resolver::normalize_path(&f);
        if unique.insert(normalized.clone()) {
            result.push(normalized);
        }
    }

    Ok(result)
}

fn get_git_changed_files(
    root: &Path,
    base: Option<&str>,
    head: Option<&str>,
) -> Result<Vec<PathBuf>> {
    let mut changed = HashSet::new();

    if let Some(base_commit) = base {
        let head_commit = head.unwrap_or("HEAD");
        let output = run_git(
            &[
                "diff",
                "--relative",
                "--name-only",
                &format!("{}...{}", base_commit, head_commit),
            ],
            root,
        )?;
        for line in output.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                changed.insert(PathBuf::from(trimmed));
            }
        }
    } else {
        // Unstaged changes
        if let Ok(output) = run_git(&["diff", "--relative", "--name-only"], root) {
            for line in output.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    changed.insert(PathBuf::from(trimmed));
                }
            }
        }
        // Staged changes
        if let Ok(output) = run_git(&["diff", "--cached", "--relative", "--name-only"], root) {
            for line in output.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    changed.insert(PathBuf::from(trimmed));
                }
            }
        }
        // Untracked changes
        if let Ok(output) = run_git(&["ls-files", "--others", "--exclude-standard"], root) {
            for line in output.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    changed.insert(PathBuf::from(trimmed));
                }
            }
        }
    }

    let mut result: Vec<_> = changed.into_iter().collect();
    result.sort();
    Ok(result)
}

fn run_git(args: &[&str], root: &Path) -> Result<String> {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .output()?;
    if !output.status.success() {
        anyhow::bail!(
            "git command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8(output.stdout)?)
}

fn is_global_config_path(path: &str) -> bool {
    if matches!(
        path,
        "package.json"
            | "pnpm-lock.yaml"
            | "package-lock.json"
            | "yarn.lock"
            | "tsconfig.json"
            | ".no-mistakes.yml"
            | ".no-mistakes.yaml"
    ) {
        return true;
    }

    matches!(
        Path::new(path).file_name().and_then(|name| name.to_str()),
        Some(
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
        )
    )
}

fn discover_all_tests(
    root: &Path,
    config: &no_mistakes_core::config::v2::NoMistakesConfig,
) -> Result<Vec<PathBuf>> {
    let filter = TestFileFilter::new(root, config);
    Ok(no_mistakes_core::codebase::ts_source::discover_files(
        root,
        &config.filesystem.skip_directories,
    )
    .into_iter()
    .filter(|f| filter.is_match(root, f))
    .collect())
}

fn slash_node_name(node: &NodeId, root: &Path) -> String {
    match node {
        NodeId::File(p) => no_mistakes_core::codebase::ts_source::relative_slash_path(root, p),
        NodeId::QueueJob { queue_file, job } => {
            let rel = no_mistakes_core::codebase::ts_source::relative_slash_path(root, queue_file);
            format!("{}#{}", rel, job)
        }
    }
}

fn relative_path(root: &Path, absolute: &Path) -> String {
    no_mistakes_core::codebase::ts_source::relative_slash_path(root, absolute)
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
                if !visited.contains(neighbor) {
                    visited.insert(neighbor.clone());
                    parents.insert(neighbor.clone(), (current.clone(), *kind));
                    queue.push_back(neighbor.clone());
                }
            }
        }
    }

    (reachable, parents)
}

fn path_confidence(edges: &[EdgeKind]) -> Confidence {
    let mut conf = Confidence::High;
    for edge in edges {
        match edge {
            EdgeKind::HttpCall
            | EdgeKind::ProcessSpawn
            | EdgeKind::QueueEnqueue
            | EdgeKind::QueueWorker
            | EdgeKind::RouteRef
            | EdgeKind::Layout
            | EdgeKind::RouteTest => return Confidence::Low,
            EdgeKind::DynamicImport => conf = Confidence::Medium,
            _ => {}
        }
    }
    conf
}

fn impact_reason_label(edge: EdgeKind) -> &'static str {
    match edge {
        EdgeKind::Import
        | EdgeKind::TypeImport
        | EdgeKind::DynamicImport
        | EdgeKind::Require
        | EdgeKind::WorkspaceImport => "dependency",
        EdgeKind::RouteRef | EdgeKind::RouteTest => "route",
        EdgeKind::Layout => "layout",
        EdgeKind::TestOf => "test",
        EdgeKind::QueueEnqueue | EdgeKind::QueueWorker => "queue",
        EdgeKind::MarkdownLink => "md",
        EdgeKind::CiInvocation => "ci",
        EdgeKind::HttpCall => "http",
        EdgeKind::ProcessSpawn => "process",
    }
}
