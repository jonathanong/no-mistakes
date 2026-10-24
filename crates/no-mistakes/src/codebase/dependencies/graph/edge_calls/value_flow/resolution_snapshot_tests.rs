use super::*;
use std::cell::Cell;
// Counts are (admitted snapshots, shallow vector backing bytes, primary site copies).
// Nested alternative sites and their dynamic strings/paths are excluded from these budgets.
thread_local! { static RETAINED: Cell<(usize, usize, usize)> = const { Cell::new((0, 0, 0)) }; }
pub(super) fn record(count: usize, bytes: usize, primary_copies: usize) {
    RETAINED.set((count, bytes, primary_copies));
}

fn with_fixture(
    run: impl FnOnce(&GraphEdgeBuildInputs<'_>, &dyn TsFactLookup, &dyn ImportResolution),
) {
    use crate::codebase::ts_source::facts::{TsFactPlan, collect_ts_facts};
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/value-flow-replacement-scale-16/fixture");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let paths = vec![root.join("src/entry.mts")];
    let facts = collect_ts_facts(
        &paths,
        TsFactPlan {
            imports: true,
            function_calls: true,
            value_flow: true,
            ..Default::default()
        },
    );
    let files = GraphFiles::from_files(paths);
    let config = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let resolver = crate::codebase::ts_resolver::ImportResolver::new(&config);
    let inputs = GraphEdgeBuildInputs {
        root: &root,
        tsconfig: &config,
        tsconfig_catalog: None,
        plan: GraphBuildPlan {
            imports: true,
            calls: true,
            ..Default::default()
        },
        workspace: None,
        graph_files: &files,
        config_options: None,
        playwright_settings: &[],
        config_path: None,
        dotnet_facts: None,
        swift_facts: None,
        import_resolution_cache: None,
        visible_paths: None,
        workflow_documents: None,
        interner: std::sync::Arc::new(PathInterner::new()),
    };
    run(&inputs, &facts, &resolver);
}

fn run(cloning: bool) -> (Vec<ResolvedCallSite>, Vec<Edge>, (usize, usize, usize)) {
    super::super::super::site_storage::with_cloning(cloning, || {
        let mut result = None;
        with_fixture(|inputs, facts, resolver| {
            let pass =
                collect_call_edges_for_core(inputs, facts, resolver, &mut fx_map(), &mut fx_map());
            result = Some((pass.sites, pass.edges, RETAINED.get()));
        });
        result.unwrap()
    })
    .0
}

#[test]
fn admitted_resolution_snapshots_match_owned_oracle_without_primary_site_copies() {
    let old_snapshot = std::env::var_os("NO_MISTAKES_RESOLUTION_SNAPSHOT_ORIGINAL").is_some();
    let (actual_sites, actual_edges, actual) = run(old_snapshot);
    let (expected_sites, expected_edges, original) = run(true);
    println!("RESOLUTION_SNAPSHOT_WORK actual={actual:?} original={original:?}");
    assert_eq!(actual_sites, expected_sites);
    assert_eq!(actual_edges, expected_edges);
    assert_eq!(actual_sites.len(), 16);
    assert_eq!(
        original.2, 16,
        "original snapshot copies every primary occurrence"
    );
    assert_eq!(actual.2, 0, "primary metadata stays in the canonical sites");
    assert!(actual.0 > 0);
    assert!(
        actual.1 < original.1,
        "compact dense shallow snapshot backing must shrink"
    );
}

#[test]
fn admission_owns_only_selected_initial_resolution_and_resets_all_fields() {
    with_fixture(|inputs, facts, resolver| {
        let pass =
            collect_call_edges_for_core(inputs, facts, resolver, &mut fx_map(), &mut fx_map());
        let mut sites = pass.sites;
        let initial = &mut sites[0];
        // Alternative targets need their original deduplicated canonical nodes too.
        initial.target =
            ResolvedCallTarget::Alternatives(std::sync::Arc::new(ResolvedCallAlternatives::new(
                vec![ResolvedCallCandidate {
                    target: KnownCallTarget::RepositoryFunction {
                        file: initial.file.clone(),
                        scope: "alternative".into(),
                    },
                    node: Some(NodeId::symbol(&initial.file, "alternative")),
                }],
                true,
            )));
        initial.target_node = None;
        initial.alternatives = vec![initial.clone()];
        initial.typed_from = vec![("typed".into(), "export".into())];
        initial.returned_from = vec![("factory".into(), "export".into())];
        initial.returned_member_from = vec![("member".into(), "export".into(), "method".into())];
        let expected = initial.clone();
        let pending = [0].into_iter().collect();
        let base = InitialResolutions::new(&sites, &pending);
        assert_eq!(base.values.len(), 1);
        assert_eq!(
            base.get(0).target_nodes().collect::<Vec<_>>(),
            expected.target_nodes().collect::<Vec<_>>()
        );
        assert_eq!(
            base.slots
                .iter()
                .filter(|&&slot| slot != usize::MAX)
                .count(),
            1
        );
        for _ in 0..2 {
            sites[0].target = ResolvedCallTarget::Unknown;
            sites[0].target_node = None;
            sites[0].alternatives.clear();
            sites[0].typed_from.clear();
            sites[0].returned_from.clear();
            sites[0].returned_member_from.clear();
            reset_resolution(&mut sites[0], base.get(0));
            assert_eq!(
                sites[0], expected,
                "each visit restores the immutable initial resolution"
            );
        }
        let empty = InitialResolutions::new(&sites, &fx_set());
        assert_eq!(empty.values.capacity(), 0);
        assert_eq!(empty.slots.capacity(), 0);
    });
}
