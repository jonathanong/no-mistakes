fn constructor_resolution_fixture(name: &str) -> PathBuf {
    category_fixture("rules", &format!("unconstructed-error-class/{name}"))
}

fn constructor_resolution_targets(report: &Value, channel: &str) -> Vec<String> {
    report[channel]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            format!(
                "{}:{} {}",
                row["file"].as_str().unwrap(),
                row["line"],
                row["target"].as_str().unwrap()
            )
        })
        .collect()
}

#[test]
fn constructor_resolution_check_and_analyze_project_preserve_proof_and_recall() {
    let root = constructor_resolution_fixture("shared-package-targets");
    for (config, advisories) in [
        (".no-mistakes.yml", 2),
        ("advise.yml", 2),
        ("ignore.yml", 0),
    ] {
        let report = recall_check_report(&root, config);
        assert_eq!(
            constructor_resolution_targets(&report, "rules"),
            ["src/errors.ts:3 UnusedError", "src/errors.ts:5 UnusedTwin",],
            "{report}"
        );
        assert_eq!(
            report["advisories"].as_array().unwrap().len(),
            advisories,
            "{report}"
        );
        assert!(
            report["advisories"].as_array().unwrap().iter().all(|row| {
                matches!(
                    row["file"].as_str(),
                    Some("src/mutation-before.ts" | "src/opaque.ts")
                )
            }),
            "{report}"
        );
    }
    let report = recall_check_report(&root, "positive-only.yml");
    assert_eq!(report["rules"], json!([]), "{report}");
    assert_eq!(report["advisories"], json!([]), "{report}");
    assert_eq!(report["warnings"], json!([]), "{report}");
}

#[test]
fn constructor_resolution_ambient_proof_matches_standalone_without_hiding_unknowns() {
    let root = constructor_resolution_fixture("ambient-module-targets");
    let report = recall_check_report(&root, ".no-mistakes.yml");
    assert_eq!(
        constructor_resolution_targets(&report, "rules"),
        ["src/main.ts:3 UnusedError"],
        "{report}"
    );
    assert_eq!(
        constructor_resolution_targets(&report, "advisories"),
        [
            "src/opaque.ts:5 unanalyzable",
            "src/opaque.ts:6 unanalyzable",
            "src/opaque.ts:7 unanalyzable",
            "src/opaque.ts:8 unanalyzable",
        ],
        "{report}"
    );
}

#[test]
fn constructor_resolution_preserves_ordinary_queries_in_both_request_orders() {
    for (name, symbol, export, consumer, proof) in [
        (
            "shared-package-targets",
            "src/ordinary.ts",
            "ordinary",
            "src/ordinary-use.ts",
            "node_modules/pg/index.d.ts",
        ),
        (
            "workspace-export-targets",
            "app/ordinary.ts",
            "ordinary",
            "app/ordinary-use.ts",
            "packages/cache-core/dynamic-config.mts",
        ),
    ] {
        constructor_resolution_ordinary_parity(name, symbol, export, consumer, proof);
    }
}

fn constructor_resolution_ordinary_parity(
    name: &str,
    symbol_file: &str,
    export_name: &str,
    import_file: &str,
    proof_file: &str,
) {
    let source = constructor_resolution_fixture(name);
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let standalone = vec![
        parse_json(
            crate::napi_api::symbols_json_impl(crate::napi_api::options::test_json_arg(
                json!({"root":root,"files":[symbol_file]}),
            ))
            .unwrap(),
        ),
        parse_json(
            crate::napi_api::dependencies_json_impl(crate::napi_api::options::test_json_arg(
                json!({
                    "root":root,"files":[import_file],"relationships":["import"]
                }),
            ))
            .unwrap(),
        ),
        ordinary_call_sites(&root, symbol_file, export_name),
    ];
    assert!(!standalone[0]["files"][0]["exports"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(!standalone[1]["files"].as_array().unwrap().is_empty());
    assert!(!standalone[2]["callSites"].as_array().unwrap().is_empty());
    let check = recall_check_report(&root, ".no-mistakes.yml");
    for check_first in [false, true] {
        let mut reports = vec![
            json!({"type":"symbols","files":[symbol_file]}),
            json!({"type":"dependencies","files":[import_file],"relationships":["import"]}),
            json!({"type":"callSites","file":symbol_file,"exportName":export_name}),
        ];
        reports.insert(if check_first { 0 } else { 3 }, json!({"type":"check"}));
        crate::ast::begin_parse_count(&root);
        let (results, observer) = observed_call_batch(&root, json!(reports));
        let parses = crate::ast::finish_parse_count(&root);
        assert!(!parses.is_empty());
        assert!(parses.values().all(|count| *count == 1), "{parses:#?}");
        let offset = usize::from(check_first);
        assert_eq!(&results[offset..offset + 3], standalone.as_slice());
        assert_eq!(results[if check_first { 0 } else { 3 }], check);
        let reads = observer.source_read_snapshot();
        assert!(!reads.is_empty());
        assert!(reads.values().all(|count| *count == 1), "{reads:#?}");
        assert_eq!(reads.get(&root.join(symbol_file)), Some(&1));
        assert_eq!(reads.get(&root.join(proof_file)), Some(&1));
        assert_eq!(observer.snapshot().work["discovery.roots"], 1);
    }
}

#[test]
fn constructor_resolution_complete_declarations_remove_legacy_package_noise_in_node() {
    let root = constructor_resolution_fixture("real-imports");
    let report = recall_check_report(&root, ".no-mistakes.yml");
    assert_eq!(
        constructor_resolution_targets(&report, "rules"),
        ["src/errors.ts:1 UnusedError"],
        "{report}"
    );
    assert_eq!(report["advisories"], json!([]), "{report}");
}

#[test]
fn constructor_resolution_workspace_exports_match_check_and_analyze_project() {
    let root = constructor_resolution_fixture("workspace-export-targets");
    let report = recall_check_report(&root, ".no-mistakes.yml");
    assert_eq!(
        constructor_resolution_targets(&report, "rules"),
        ["app/queues/errors.mts:3 UnusedError"],
        "{report}"
    );
    assert_eq!(report["advisories"], json!([]), "{report}");
    let positive = recall_check_report(&root, "positive-only.yml");
    assert_eq!(positive["rules"], json!([]), "{positive}");
    assert_eq!(positive["advisories"], json!([]), "{positive}");
}
