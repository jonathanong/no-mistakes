fn signature_parser_count_fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/parser-count/signature-impact"),
    )
}

#[test]
fn file_backed_symbol_workers_report_absolute_parse_paths() {
    let source = signature_parser_count_fixture_root();
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let files = ["consumer.mts", "consumer.test.mts", "utils.mts"];
    let args = args_for(&root, files.to_vec(), Format::Json);

    crate::ast::begin_parse_count(&root);
    let (entries, _) = collect_entries(&args).unwrap();
    let counts = crate::ast::finish_parse_count(&root);

    assert_eq!(entries.len(), files.len());
    assert_eq!(counts.len(), files.len(), "{counts:?}");
    // These parses happen on Rayon workers. Keeping exact absolute keys here
    // prevents a source-only sentinel from hiding or cross-attributing them.
    for file in files {
        assert_eq!(counts.get(&root.join(file)), Some(&1), "{counts:?}");
    }
    assert!(!counts.contains_key(Path::new("symbols.ts")), "{counts:?}");
    assert!(!counts.contains_key(Path::new("symbols.tsx")), "{counts:?}");
}

#[test]
fn signature_impact_parses_each_source_file_once() {
    let source = signature_parser_count_fixture_root();
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let mut args = args_for(&root, vec!["utils.mts"], Format::Json);
    args.mode = SymbolsMode::SignatureImpact;
    args.symbol = Some("parseDate".to_string());

    crate::ast::begin_parse_count(&root);
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let output = {
        let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
        impact::report_json(args).unwrap()
    };
    let counts = crate::ast::finish_parse_count(&root);
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    let expected = [
        root.join("consumer.mts"),
        root.join("consumer.test.mts"),
        root.join("utils.mts"),
    ];

    assert!(report["productionCallers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["file"] == "consumer.mts"));
    assert!(report["suggestedTests"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["file"] == "consumer.test.mts"));
    assert_eq!(counts.len(), expected.len(), "{counts:?}");
    assert!(counts.values().all(|count| *count == 1), "{counts:?}");
    for file in expected {
        assert_eq!(counts.get(&file), Some(&1), "{counts:?}");
    }
    // This fixture intentionally has no package manifests, so only the config
    // and tsconfig gateways perform manifest work. Workspace preparation must
    // be threaded into reporting instead of issuing cache-hit reloads.
    let work = observer.snapshot().work;
    assert_eq!(work["manifest.requests"], 2, "{work:#?}");
    assert_eq!(work["manifest.parses"], 2, "{work:#?}");
    assert_eq!(
        work.get("manifest.cache_hits").copied().unwrap_or_default(),
        0,
        "{work:#?}",
    );
    let source_reads = observer.source_read_snapshot();
    assert!(
        source_reads.values().all(|reads| *reads == 1),
        "each request source must be physically read once: {source_reads:#?}"
    );
}
