use super::*;
use crate::config::v2::NoMistakesConfig;

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
