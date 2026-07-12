use super::*;
use crate::config::v2::NoMistakesConfig;

#[test]
fn caller_entries_filters_export_nodes_and_non_file_nodes() {
    let root = Path::new("/repo");
    let source = PathBuf::from("/repo/src/source.mts");
    let consumer = PathBuf::from("/repo/src/consumer.mts");
    let test = PathBuf::from("/repo/src/consumer.test.mts");
    let export_node = NodeId::Symbol {
        file: source,
        symbol: "parseDate".to_string(),
    };
    let entries = vec![
        NodeEntry {
            node: export_node.clone(),
            depth: 1,
            via: vec![EdgeKind::Import],
        },
        NodeEntry {
            node: NodeId::Module("external".to_string()),
            depth: 1,
            via: vec![EdgeKind::Import],
        },
        NodeEntry {
            node: NodeId::Symbol {
                file: consumer,
                symbol: "format".to_string(),
            },
            depth: 1,
            via: vec![EdgeKind::Import],
        },
        NodeEntry {
            node: NodeId::File(test),
            depth: 1,
            via: vec![EdgeKind::TestOf],
        },
    ];
    let filter = TestFileFilter::new(root, &NoMistakesConfig::default());
    let export_nodes = BTreeSet::from([export_node]);
    let file_target_symbols = BTreeMap::new();
    let context = CallerEntriesContext {
        root,
        test_filter: &filter,
        export_nodes: &export_nodes,
        file_target_symbols: &file_target_symbols,
    };

    let production = caller_entries(&entries, &context, false, &[]);
    let tests = caller_entries(&entries, &context, true, &[]);

    assert_eq!(production.len(), 1);
    assert_eq!(production[0].file, "src/consumer.mts");
    assert_eq!(production[0].symbol.as_deref(), Some("format"));
    assert!(tests.is_empty());
}

#[test]
fn caller_entries_merges_duplicate_callers_and_sorts() {
    let root = Path::new("/repo");
    let filter = TestFileFilter::new(root, &NoMistakesConfig::default());
    let export_nodes = BTreeSet::new();
    let file_target_symbols = BTreeMap::new();
    let context = CallerEntriesContext {
        root,
        test_filter: &filter,
        export_nodes: &export_nodes,
        file_target_symbols: &file_target_symbols,
    };
    let entries = vec![
        NodeEntry {
            node: NodeId::Symbol {
                file: PathBuf::from("/repo/src/b.mts"),
                symbol: "beta".to_string(),
            },
            depth: 1,
            via: vec![EdgeKind::Import],
        },
        NodeEntry {
            node: NodeId::Symbol {
                file: PathBuf::from("/repo/src/a.mts"),
                symbol: "alpha".to_string(),
            },
            depth: 1,
            via: vec![EdgeKind::Import],
        },
        NodeEntry {
            node: NodeId::Symbol {
                file: PathBuf::from("/repo/src/b.mts"),
                symbol: "beta".to_string(),
            },
            depth: 1,
            via: vec![EdgeKind::Import],
        },
    ];

    let extra = vec![CallerEntry {
        file: "src/a.mts".to_string(),
        symbol: Some("alpha".to_string()),
        depth: 2,
        via: vec!["symbol"],
    }];
    let callers = caller_entries(&entries, &context, false, &extra);

    assert_eq!(callers.len(), 2);
    assert_eq!(callers[0].file, "src/a.mts");
    assert_eq!(callers[0].via, vec!["import", "symbol"]);
    assert_eq!(callers[1].file, "src/b.mts");
    assert_eq!(callers[1].depth, 1);
    assert_eq!(callers[1].via, vec!["import"]);
}

#[test]
fn file_entry_uses_symbol_checks_extracted_and_alias_member_uses() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/tests-impact-symbol/fixture");

    assert!(file_entry_uses_symbol(
        &root,
        "require-caller.mts",
        "parseDate"
    ));
    assert!(file_entry_uses_symbol(
        &root,
        "dynamic-import-caller.mts",
        "parseDate"
    ));
    assert!(file_entry_uses_symbol(
        &root,
        "dynamic-import-alias-caller.mts",
        "parseDate"
    ));
    assert!(!file_entry_uses_symbol(
        &root,
        "dynamic-import-unused.mts",
        "parseDate"
    ));
    assert!(!file_entry_uses_symbol(
        &root,
        "dynamic-import-shadowed-member.mts",
        "parseDate"
    ));
    assert!(!file_entry_uses_symbol(
        &root,
        "dynamic-import-other-export-name.mts",
        "parseDate"
    ));
    assert!(file_entry_uses_symbol(
        &root,
        "dynamic-import-chained-member-caller.mts",
        "parseDate"
    ));
    assert!(!file_entry_uses_symbol(
        &root,
        "missing-dynamic-import-caller.mts",
        "parseDate"
    ));
}

#[test]
fn symbol_aliases_collect_destructured_and_member_assignment_locals() {
    let aliases = dynamic_symbol_aliases_in_source(
        "const { parseDate: pd } = await import('./utils.mts');\n\
         const readDate = require('./utils.mts').parseDate;\n\
         return utils.parseDate;\n\
         assigned = utils.parseDate;\n\
         pd(value); readDate(value);",
        "parseDate",
    );

    assert!(aliases.contains("pd"));
    assert!(aliases.contains("readDate"));
    assert!(!aliases.contains("assigned"));
}

#[test]
fn dynamic_usage_helpers_ignore_non_module_and_malformed_bindings() {
    assert!(dynamic_module_bindings("const utils = await import('./utils.mts');").contains("utils"));
    assert!(dynamic_module_bindings("const readDate = require('./utils.mts').parseDate;").is_empty());
    assert!(destructured_symbol_aliases("const { parseDate = await import('./utils.mts');", "parseDate").is_empty());
    assert!(member_assignment_alias("exports.value = utils.parseDate;", "parseDate").is_empty());
    assert!(member_assignment_alias("const readDate = utils.other;", "parseDate").is_empty());
    assert!(dynamic_symbol_aliases_in_source("const utils = await import('./utils.mts');", "dates.parseDate").is_empty());
    assert!(source_contains_member_name("utils.parseDate(value)", "utils.parseDate"));
    assert!(!source_contains_member_name(
        "utils.parseDateOld(value)",
        "utils.parseDate"
    ));
    assert!(source_contains_call_name("pd(value)", "pd"));
    assert!(source_contains_call_name("pd (value)", "pd"));
    assert!(!source_contains_call_name("otherpd(value)", "pd"));
}

#[test]
fn local_callee_matching_accepts_namespace_members() {
    assert!(matches_local_callee(
        "dates.parseDate",
        &BTreeSet::from(["dates".to_string()])
    ));
    assert!(matches_local_callee(
        "parseDate",
        &BTreeSet::from(["parseDate".to_string()])
    ));
    assert!(!matches_local_callee(
        "updatedDates.parseDate",
        &BTreeSet::from(["dates".to_string()])
    ));
}

/// Regression test for the `prepare_local_caller_context` hoist: it must resolve the workspace
/// map from the same `.gitignore`-aware file list the dependency graph itself uses
/// (`ts_source::discover_files`), not from `graph.all_files()`.
///
/// `graph.all_files()` only yields indexable TS/JS graph nodes, which never include
/// `package.json`. Feeding that narrower list into `workspaces::load_from_files` silently
/// resolves zero workspace packages, which regressed
/// `signature_impact_recovers_workspace_import_private_callers` (a black-box symptom of this
/// exact bug) when first tried during this fix. This test pins the fix at the unit level so a
/// future refactor that re-derives the workspace file list from the graph fails fast here.
#[test]
fn prepare_local_caller_context_resolves_workspace_packages_once() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/tests-impact-symbol/fixture"),
    );
    let tsconfig = crate::codebase::ts_resolver::resolve_tsconfig(None, &root).unwrap();
    let graph = DepGraph::build_with_plan_and_config(
        &root,
        &tsconfig,
        signature_impact_graph_plan(),
        None,
    )
    .unwrap();

    let context = prepare_local_caller_context(&graph, &root);

    assert!(
        !context.workspace.packages.is_empty(),
        "workspace packages must be resolved from a package.json-inclusive file list"
    );
    assert!(context.workspace.resolve_package("@repo/dates").is_some());
    assert!(!context.facts.is_empty());

    // The same prepared context must serve both `want_tests` passes without recomputing facts
    // or the workspace map. `packages/dates/index.mts` re-exports `parseDate` from `utils.mts`,
    // so it needs a `target_symbols` entry too -- this mirrors what `signature_target_symbols`
    // produces in the real `collect_report` pipeline, where every file on the re-export chain
    // (including workspace package entry points) gets an entry.
    let target_symbols = BTreeMap::from([
        (root.join("utils.mts"), BTreeSet::from(["parseDate".to_string()])),
        (
            root.join("packages/dates/index.mts"),
            BTreeSet::from(["parseDate".to_string()]),
        ),
    ]);
    let filter = TestFileFilter::new(&root, &NoMistakesConfig::default());
    let production =
        local_caller_entries(&context, &target_symbols, &root, &tsconfig, &filter, false);
    let tests = local_caller_entries(&context, &target_symbols, &root, &tsconfig, &filter, true);

    assert!(production
        .iter()
        .any(|caller| caller.file == "workspace-private-caller.mts"));
    assert!(!tests
        .iter()
        .any(|caller| caller.file == "workspace-private-caller.mts"));
}
