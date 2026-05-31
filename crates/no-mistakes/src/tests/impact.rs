use crate::tests::comment::render_markdown_plan;
use crate::tests::plan::{
    bfs_path_find, path_confidence, relative_path, slash_node_name, symbol_aware_start_nodes,
};
use crate::tests::{
    Confidence, ImpactArgs, ImpactReason, PlanFormat, SelectedTest, TestPlan, Warning,
};
use anyhow::{Context, Result};
use no_mistakes::codebase::dependencies::graph::{DepGraph, EdgeKind, NodeId};
use no_mistakes::codebase::dependencies::parse_entrypoint;
use no_mistakes::codebase::test_filter::TestFileFilter;
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

    match format {
        PlanFormat::Json => println!("{}", serde_json::to_string_pretty(&plan)?),
        PlanFormat::Paths => {
            for test in &plan.selected_tests {
                println!("{}", test.test_file);
            }
        }
        PlanFormat::Markdown | PlanFormat::Md => println!("{}", render_markdown_plan(&plan)),
    }

    Ok(ExitCode::SUCCESS)
}

const _: fn(ImpactArgs) -> Result<ExitCode> = run;

pub fn generate_impact_plan(args: &ImpactArgs) -> Result<TestPlan> {
    let cwd = std::env::current_dir().context("cwd must be accessible")?;
    let root = no_mistakes::cli::resolve_optional_root(Some(&args.root), &cwd);
    let root = no_mistakes::codebase::ts_resolver::normalize_path(&root);
    let root = root.canonicalize().unwrap_or(root);

    let config = load_v2_config(&root, args.config.as_deref())?;
    let tsconfig = crate::tests::why::resolve_tsconfig(args.tsconfig.as_deref(), &root)?;
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
        let start_nodes =
            symbol_aware_start_nodes(&graph, &normalized, symbol.as_ref(), args.include_symbols);
        let rel_changed = symbol
            .as_ref()
            .filter(|_| args.include_symbols)
            .map_or_else(
                || relative_path(&root, &normalized),
                |symbol| format!("{}#{}", relative_path(&root, &normalized), symbol),
            );

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
            };
            if !entry.reasons.contains(&reason) {
                entry.reasons.push(reason);
            }
            continue;
        }

        for start_node in start_nodes {
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

                while let Some((parent, kind)) = path_parents.get(&curr) {
                    node_chain.push(slash_node_name(parent, &root));
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
                node_chain.reverse();

                let via_strings: Vec<String> = edge_path
                    .iter()
                    .map(|k| crate::tests::plan::impact_reason_label(*k).to_string())
                    .collect();

                let reason = ImpactReason {
                    changed_file: rel_changed.clone(),
                    path: node_chain,
                    via: via_strings,
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

fn push_warning(
    root: &Path,
    curr: &NodeId,
    parent: &NodeId,
    kind: EdgeKind,
    warnings: &mut Vec<Warning>,
    warnings_seen: &mut HashSet<(String, String)>,
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
    };
    if warnings_seen.insert((warn.r#type.clone(), warn.file.clone())) {
        warnings.push(warn);
    }
}
