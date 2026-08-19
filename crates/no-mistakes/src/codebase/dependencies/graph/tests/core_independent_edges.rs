#[test]
fn collect_independent_core_edges_parallelizes_import_symbol_and_test_kinds() {
    let builder = include_str!("../builder_edges.rs");
    let independent = include_str!("../builder_core_edges_independent.rs");
    assert!(
        builder.contains("collect_independent_core_edges"),
        "core-edge orchestration must collect the independent import/symbol panel"
    );
    assert!(
        independent.contains("rayon::join"),
        "import vs route-import vs workspace/package/assets/symbols/tests must collect via rayon::join"
    );
    assert!(
        independent.contains("collect_import_edges_for_core")
            && independent.contains("collect_symbol_edges_for_core")
            && independent.contains("collect_test_edges_for_core"),
        "independent core panel must collect import, symbol, and test edges"
    );
    assert!(
        independent.contains("merge_independent_core_edges")
            && independent.contains("edges.imports")
            && independent.contains("edges.route_imports")
            && independent.contains("edges.workspace")
            && independent.contains("edges.package")
            && independent.contains("edges.assets")
            && independent.contains("edges.symbols")
            && independent.contains("edges.tests"),
        "core merge order must stay imports, route_imports, workspace, package, assets, symbols, tests"
    );
    assert!(
        independent.contains("traced_parallel_edges")
            && independent.contains("TimingKind::Parallel"),
        "independent join leaves must mark overlapping work as Parallel"
    );
    assert!(
        !builder.contains("merge_edges(forward, reverse, import_edges)"),
        "core kinds must not merge on the rayon worker that collected them"
    );
}
