use super::*;

fn fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/foreign-receiver-work-budget/fixture"),
    )
}

fn check(config: &str) -> serde_json::Value {
    let root = fixture_root();
    serde_json::from_str(
        &check_json_impl(json!({
            "root": root,
            "config": root.join(config),
        }))
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn exhausted_value_flow_keeps_complete_import_only_dependency_violations() {
    let imports = check("import-only-config.yml");
    let mixed = check("import-mixed-config.yml");
    let select = |report: &serde_json::Value| {
        report["rules"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|finding| finding["rule"] == "forbidden-dependencies")
            .cloned()
            .collect::<Vec<_>>()
    };
    let baseline = select(&imports);
    assert!(
        !baseline.is_empty(),
        "saved import projection must prove a violation: {imports}"
    );
    assert_eq!(baseline, select(&mixed));
    let failures = mixed["incomplete"].as_array().unwrap();
    assert!(failures
        .iter()
        .any(|failure| failure["consumer"] == "unconstructed-error-class"));
    assert!(!failures
        .iter()
        .any(|failure| failure["consumer"] == "forbidden-dependencies"));
}

#[test]
fn every_exhausted_graph_only_file_has_its_own_repair_hint() {
    let root = fixture_root();
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join("graph-only-config.yml")))
            .unwrap();
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let views = crate::check_discovery::discover_check_file_views_from_snapshot(
        &root,
        &config,
        &config.filesystem.skip_directories,
        false,
        &snapshot,
    );
    for file in ["shared-2048.mts", "shared-4096.mts"] {
        assert!(!views.filesystem.iter().any(|path| path.ends_with(file)));
        assert!(views.graph.iter().any(|path| path.ends_with(file)));
    }
    let report = check("graph-only-config.yml");
    let failures = report["incomplete"].as_array().unwrap();
    for file in ["shared-2048.mts", "shared-4096.mts"] {
        assert!(
            failures.iter().any(|failure| {
                failure["consumer"] == "unconstructed-error-class"
                    && failure["file"]
                        .as_str()
                        .is_some_and(|path| PathBuf::from(path).ends_with(file))
                    && failure["message"].as_str().is_some_and(|message| {
                        message.contains("per-file work budget")
                            && message.contains("narrower analysis root")
                    })
            }),
            "missing repair hint for graph-only {file}: {report}"
        );
    }
    assert!(!report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| finding["rule"] == "unconstructed-error-class"));
}
