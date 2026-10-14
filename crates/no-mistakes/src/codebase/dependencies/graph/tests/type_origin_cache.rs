use super::*;

#[test]
fn independent_reexported_type_origins_preserve_parallel_graph_output() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("type-origin-cache"));
    let config = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let build = |threads| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap()
            .install(|| {
                DepGraph::build_with_plan(
                    &root,
                    &config,
                    GraphBuildPlan {
                        calls: true,
                        effect_selectors: true,
                        ..Default::default()
                    },
                )
                .unwrap()
            })
    };
    let serial = build(1);
    let parallel = build(8);
    for index in 0..16 {
        let path = root.join(format!("helpers{index}.mts"));
        let expected = serial.call_sites_in_file(&path);
        let actual = parallel.call_sites_in_file(&path);
        assert_eq!(format!("{actual:?}"), format!("{expected:?}"));
        assert_eq!(
            actual
                .iter()
                .filter(|site| matches!(&site.target,
                    ResolvedCallTarget::ModuleExport { export_path, .. }
                    if export_path == "primary.get"))
                .count(),
            128
        );
    }
}
