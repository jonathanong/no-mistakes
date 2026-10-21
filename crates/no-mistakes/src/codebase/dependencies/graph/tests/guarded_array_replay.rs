use super::*;

#[test]
fn guarded_array_stress_retains_early_and_distinct_late_targets() {
    let root =
        crate::codebase::ts_resolver::normalize_path(&fixture("value-flow-guarded-array-replay"));
    let config = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let built = DepGraph::build_with_plan(
        &root,
        &config,
        GraphBuildPlan {
            calls: true,
            ..Default::default()
        },
    )
    .unwrap();
    for file in [
        "same-width",
        "varying-width",
        "repeated-spreads",
        "trailing-arguments",
        "either-array",
        "method-consumer",
    ] {
        for callee in ["replica.get", "tail.get", "untouched.get"] {
            let sites = built.call_sites_in_file(&root.join(format!("src/{file}.mts")));
            let site = sites
                .iter()
                .find(|site| site.source_callee == callee)
                .unwrap();
            assert!(site.has_unknown_target(), "{file}: {site:?}");
            assert_eq!(site.targets().count(), 1, "{file}: {site:?}");
            assert!(
                site.targets().any(|candidate| matches!(candidate.target,
                    KnownCallTargetRef::ModuleExport { specifier: "@vendor/valkey", export_path, .. } if export_path == callee
                )),
                "{file}: {site:?}"
            );
        }
        for callee in ["replica.info", "unrelated.get"] {
            receiver_escapes::assert_export(&built, &root, file, callee);
        }
    }
    receiver_escapes::assert_unknown(&built, &root, "interleaved", "tail.get");
    assert_possible(&built, &root, "interleaved", "untouched.get");
    for callee in ["replica.info", "unrelated.get"] {
        receiver_escapes::assert_export(&built, &root, "interleaved", callee);
    }
    let sites = built.call_sites_in_file(&root.join("src/consumer-generations.mts"));
    for callee in ["replica.get", "replica.info", "replica.del", "primary.set"] {
        let site = sites
            .iter()
            .find(|site| site.source_callee == callee)
            .unwrap();
        assert!(site.has_unknown_target(), "{site:?}");
        assert_eq!(site.targets().count(), 1, "{site:?}");
        assert!(site.targets().any(|candidate| matches!(candidate.target,
            KnownCallTargetRef::ModuleExport { specifier: "@vendor/valkey", export_path, .. } if export_path == callee
        )), "{site:?}");
    }
    receiver_escapes::assert_export(&built, &root, "consumer-generations", "replica.set");
    assert_possible(&built, &root, "consumer-generations", "untouched.get");
    receiver_escapes::assert_export(&built, &root, "consumer-generations", "unrelated.get");
    receiver_escapes::assert_export(&built, &root, "constructor-and-call", "replica.get");
    assert_possible(&built, &root, "constructor-and-call", "untouched.info");
    receiver_escapes::assert_export(&built, &root, "constructor-and-call", "unrelated.get");
    let sites = built.call_sites_in_file(&root.join("src/constructor-and-call.mts"));
    let site = sites
        .iter()
        .find(|site| site.source_callee == "replica.info")
        .unwrap();
    assert!(site.has_unknown_target(), "{site:?}");
    assert_eq!(site.targets().count(), 1, "{site:?}");
    assert!(
        site.targets().any(|candidate| matches!(
            candidate.target,
            KnownCallTargetRef::ModuleExport {
                specifier: "@vendor/valkey",
                export_path: "replica.info",
                ..
            }
        )),
        "{site:?}"
    );
}

fn assert_possible(built: &DepGraph, root: &Path, file: &str, callee: &str) {
    let sites = built.call_sites_in_file(&root.join(format!("src/{file}.mts")));
    let site = sites
        .iter()
        .find(|site| site.source_callee == callee)
        .unwrap();
    assert!(site.has_unknown_target(), "{file}: {site:?}");
    assert_eq!(site.targets().count(), 1, "{file}: {site:?}");
    assert!(
        site.targets().any(|candidate| matches!(candidate.target,
        KnownCallTargetRef::ModuleExport { specifier: "@vendor/valkey", export_path, .. }
            if export_path == callee)),
        "{file}: {site:?}"
    );
}
