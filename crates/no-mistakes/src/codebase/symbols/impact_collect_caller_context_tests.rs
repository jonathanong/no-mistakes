use super::*;
use crate::config::v2::NoMistakesConfig;

/// Regression test for the `prepare_local_caller_context` hoist: it must resolve the workspace
/// map from the same `.gitignore`-aware file list the dependency graph itself uses
/// (`GraphFiles::all()`, backed by `ts_source::discover_files`), not from the narrower
/// indexable-file set backing `facts`.
///
/// The narrower set only yields indexable TS/JS graph nodes, which never include
/// `package.json`. Feeding that narrower list into `workspaces::load_from_files` silently
/// resolves zero workspace packages, which regressed
/// `signature_impact_recovers_workspace_import_private_callers` (a black-box symptom of this
/// exact bug) when first tried during this fix. This test pins the fix at the unit level so a
/// future refactor that re-derives the workspace file list from `facts` fails fast here.
///
/// This also exercises the shared-facts/shared-discovery hand-off `collect_report` performs in
/// production: the same `TsFactMap` and `GraphFiles::all()` list built here for the `DepGraph`
/// build are moved/passed into `prepare_local_caller_context` instead of being recomputed.
#[test]
fn prepare_local_caller_context_resolves_workspace_packages_once() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/tests-impact-symbol/fixture"),
    );
    let tsconfig = crate::codebase::ts_resolver::resolve_tsconfig(None, &root).unwrap();
    let graph_plan = signature_impact_graph_plan();
    let graph_files = GraphFiles::discover(&root);
    let (fact_plan, fact_context) =
        ts_fact_plan_and_context_for_plan_with_config(&root, graph_plan, None);

    let facts = crate::codebase::ts_source::facts::collect_ts_facts_with_context(
        graph_files.indexable(),
        fact_plan,
        &fact_context,
    );

    let graph = DepGraph::build_with_plan_files_config_and_facts(
        &root,
        &tsconfig,
        graph_plan,
        &graph_files,
        None,
        Some(&facts as &dyn TsFactLookup),
    )
    .expect("shared-facts graph builds");
    assert!(
        graph.all_files().count() > 0,
        "sanity check: shared facts must still let the graph build succeed"
    );

    let context = prepare_local_caller_context(facts, graph_files.all(), &root);

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

/// Regression test: `prepare_local_caller_context` must use exactly the `TsFactMap` and
/// discovery file list it's handed, never independently re-derive either via its own
/// `collect_ts_facts`/`discover_files` calls. Constructs disagreement cases
/// (`crates/CLAUDE.md`: "assert on a call count, not value equality" / "construct a case where
/// the two approaches would disagree") rather than checking output equality, which a
/// re-deriving version would also satisfy: hands in a deliberately empty `TsFactMap` (for facts)
/// and a deliberately empty discovery file list (for the workspace map), for a fixture that has
/// real, non-empty facts and a real, non-empty workspace on disk, and asserts the resulting
/// context reflects the supplied (empty) inputs, not the real files on disk.
#[test]
fn prepare_local_caller_context_never_rederives_its_supplied_inputs() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/tests-impact-symbol/fixture"),
    );

    let empty_facts = crate::codebase::ts_source::facts::TsFactMap::default();
    let context = prepare_local_caller_context(empty_facts, &[], &root);

    assert!(
        context.facts.is_empty(),
        "prepare_local_caller_context must use exactly the supplied (empty) facts map, not \
         silently re-parse the real files on disk"
    );
    assert!(
        context.workspace.packages.is_empty(),
        "prepare_local_caller_context must resolve the workspace from exactly the supplied \
         (empty) discovery file list, not silently re-run discover_files against the real, \
         package.json-inclusive file set on disk"
    );
}
