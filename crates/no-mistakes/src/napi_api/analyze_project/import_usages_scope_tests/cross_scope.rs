#[test]
fn external_import_usage_files_do_not_widen_mixed_check_findings() {
    let fixture = fixture_root();
    let root = fixture.join("report-root");
    let external = fixture.join("external-root");
    let config = fixture.join("isolation.no-mistakes.yml");
    let standalone = parse_json(
        crate::napi_api::check_json_impl(json!({ "root": root, "config": config }).to_string())
            .unwrap(),
    );
    assert!(standalone["codebase"].as_array().unwrap().is_empty());
    let expanded = parse_json(
        crate::napi_api::check_json_impl(json!({ "root": fixture, "config": config }).to_string())
            .unwrap(),
    );
    assert!(
        expanded["codebase"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["rule"] == "unique-exports"),
        "the external duplicate must be a meaningful check violation: {expanded:#}"
    );

    let observer = crate::diagnostics::InvocationObserver::new(true);
    let output = {
        let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
        parse_json(
            analyze_project_json_impl(
                json!({
                    "root": root,
                    "config": config,
                    "reports": [
                        { "type": "check" },
                        { "type": "importUsages", "scanRoots": [external] }
                    ]
                })
                .to_string(),
            )
            .unwrap(),
        )
    };

    assert_eq!(output["reports"][0]["result"], standalone);
    let import_usages = &output["reports"][1]["result"];
    assert_eq!(import_usages["files"].as_array().unwrap().len(), 1);
    assert_eq!(
        import_usages["files"][0]["path"],
        external.join("external.ts").display().to_string()
    );
    assert_eq!(
        import_usages["files"][0]["imports"][0]["specifier"],
        "external-package"
    );
    let work = observer.snapshot().work;
    assert_eq!(work["discovery.roots"], 2, "{work:#?}");
    assert_eq!(work["discovery.requests"], 3, "{work:#?}");
    assert_eq!(work["discovery.cache_hits"], 2, "{work:#?}");
    assert_eq!(work["source.reads"], 4, "{work:#?}");
    assert_eq!(work["parse.files"], 3, "{work:#?}");
    assert_single_reads(
        &observer,
        &[
            config,
            root.join("src/entry.ts"),
            root.join("src/helper.ts"),
            external.join("external.ts"),
        ],
    );
}

#[test]
fn all_effective_scope_snapshots_are_seeded_before_scope_preparation() {
    let fixture = fixture_root();
    let root = fixture.join("external-root");
    let report_root = fixture.join("report-root");
    let inherited = parse_json(
        crate::napi_api::import_usages_json_impl(
            json!({ "root": root, "scanRoots": [report_root] }).to_string(),
        )
        .unwrap(),
    );
    let overridden = parse_json(
        crate::napi_api::import_usages_json_impl(json!({ "root": report_root }).to_string())
            .unwrap(),
    );
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let output = {
        let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
        parse_json(
            analyze_project_json_impl(
                json!({
                    "root": root,
                    "reports": [
                        {
                            "type": "importUsages",
                            "id": "inherited-root",
                            "scanRoots": [report_root]
                        },
                        {
                            "type": "importUsages",
                            "id": "overridden-root",
                            "root": report_root
                        }
                    ]
                })
                .to_string(),
            )
            .unwrap(),
        )
    };

    assert_eq!(output["reports"][0]["result"], inherited);
    assert_eq!(output["reports"][1]["result"], overridden);
    assert_eq!(
        output["reports"][0]["result"]["files"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        output["reports"][1]["result"]["files"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let work = observer.snapshot().work;
    assert_eq!(work["discovery.roots"], 2, "{work:#?}");
    assert_eq!(work["discovery.requests"], 3, "{work:#?}");
    assert_eq!(work["discovery.cache_hits"], 3, "{work:#?}");
    assert_eq!(work["source.reads"], 3, "{work:#?}");
    assert_eq!(work["parse.files"], 3, "{work:#?}");
    assert_single_reads(
        &observer,
        &[
            root.join("external.ts"),
            report_root.join("src/entry.ts"),
            report_root.join("src/helper.ts"),
        ],
    );
}
