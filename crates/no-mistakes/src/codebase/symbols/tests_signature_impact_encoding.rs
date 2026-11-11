fn encoding_fixture(case: &str) -> tempfile::TempDir {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/symbols/signature-impact-encoding")
        .join(case);
    crate::test_support::materialize_saved_fixture(&source)
}

fn encoding_args(root: &Path) -> SymbolsArgs {
    let mut args = args_for(root, vec!["target.mts"], Format::Json);
    args.mode = SymbolsMode::SignatureImpact;
    args.symbol = Some("parseDate".to_string());
    args
}

fn normalize_encoding_snapshot(root: &Path, text: &str) -> String {
    text.replace(root.to_string_lossy().as_ref(), "<root>")
        .replace("<root>\\", "<root>/")
}

fn normalize_encoding_report(root: &Path, value: &mut serde_json::Value) {
    match value {
        serde_json::Value::String(text) => *text = normalize_encoding_snapshot(root, text),
        serde_json::Value::Array(values) => {
            for value in values {
                normalize_encoding_report(root, value);
            }
        }
        serde_json::Value::Object(values) => {
            for value in values.values_mut() {
                normalize_encoding_report(root, value);
            }
        }
        _ => {}
    }
}

#[test]
fn signature_impact_preserves_lossy_callers_and_reports_unrelated_failures() {
    let fixture = encoding_fixture("tolerated");
    let root = fixture.path().canonicalize().unwrap();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    crate::ast::begin_parse_count(&root);
    let output = {
        let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
        impact::report_json(encoding_args(&root)).unwrap()
    };
    let counts = crate::ast::finish_parse_count(&root);
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    let mut snapshot = report.clone();
    normalize_encoding_report(&root, &mut snapshot);
    insta::assert_snapshot!(
        "signature_impact_encoding",
        serde_json::to_string_pretty(&snapshot).unwrap()
    );
    let callers = report["productionCallers"].as_array().unwrap();
    for file in ["consumer.mts", "comment.mts", "string.mts"] {
        assert!(
            callers.iter().any(|caller| caller["file"] == file),
            "{report}"
        );
    }
    let unanalyzed = report["unanalyzed"].as_array().unwrap();
    let files = unanalyzed
        .iter()
        .map(|entry| entry["file"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(files, ["binary.mts", "unrelated.mts"]);
    assert!(unanalyzed
        .iter()
        .all(|entry| !entry["reason"].as_str().unwrap().is_empty()));
    assert!(unanalyzed[1]["reason"].as_str().unwrap().contains("1490"));
    assert!(report["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|warning| warning["type"] == "unanalyzed-files"));
    assert_eq!(counts.len(), 6, "{counts:?}");
    assert!(counts.values().all(|count| *count == 1), "{counts:?}");
    let reads = observer.source_read_snapshot();
    for file in [
        "target.mts",
        "consumer.mts",
        "comment.mts",
        "string.mts",
        "unrelated.mts",
        "binary.mts",
    ] {
        assert_eq!(reads.get(&root.join(file)), Some(&1), "{reads:?}");
    }
}

#[test]
fn signature_impact_rejects_potentially_related_unanalyzed_files() {
    for case in [
        "raw-target",
        "aliased-target",
        "unresolved-target",
        "broken-barrel",
        "importer-dependency",
        "opaque-barrel",
        "opaque-reexport",
        "opaque-renamed-barrel",
        "opaque-namespace-barrel",
    ] {
        let fixture = encoding_fixture(case);
        let root = fixture.path().canonicalize().unwrap();
        let error = match impact::report_json(encoding_args(&root)) {
            Ok(report) => panic!("{case}: potentially related failure returned report: {report}"),
            Err(error) => error.to_string(),
        };
        assert!(
            error.contains("signature-impact could not analyze `broken.mts`"),
            "{case}: {error}"
        );
    }
}

include!("tests_signature_impact_encoding_text.rs");

#[test]
fn signature_impact_rejects_target_parse_failure_before_export_projection() {
    let fixture = encoding_fixture("invalid-target");
    let root = fixture.path().canonicalize().unwrap();
    let error = impact::report_json(encoding_args(&root)).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("signature-impact could not analyze `target.mts`"),
        "{error}"
    );
}

#[cfg(unix)]
#[test]
fn signature_impact_rejects_unreadable_sources_without_proven_independence() {
    use std::os::unix::fs::PermissionsExt;
    struct RestorePermissions(PathBuf, std::fs::Permissions);
    impl Drop for RestorePermissions {
        fn drop(&mut self) {
            std::fs::set_permissions(&self.0, self.1.clone()).unwrap();
        }
    }
    for file in ["bridge.mts", "standalone.mts", "unrelated.mts"] {
        let fixture = encoding_fixture("unreadable-renamed-bridge");
        let root = fixture.path().canonicalize().unwrap();
        let path = root.join(file);
        let restore = RestorePermissions(
            path.clone(),
            std::fs::metadata(&path).unwrap().permissions(),
        );
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).unwrap();
        if std::fs::read(&path).is_ok() {
            drop(restore);
            return;
        }
        let result = impact::report_json(encoding_args(&root));
        drop(restore);
        let error = result.unwrap_err().to_string();
        assert!(
            error.contains(&format!("signature-impact could not analyze `{file}`")),
            "{error}"
        );
        assert!(error.contains("failed to read"), "{error}");
    }
}
