use super::*;

#[test]
fn target_collection_preserves_each_request_validation_error() {
    for (helper, report, expected) in [
        (
            "symbols",
            json!({"type":"symbols", "files":false}),
            "invalid type",
        ),
        (
            "legacy",
            json!({"type":"callSites", "file":false, "exportName":"a"}),
            "invalid type",
        ),
        (
            "legacy",
            json!({"type":"symbols", "files":false}),
            "invalid type",
        ),
        (
            "legacy",
            json!({"type":"symbols", "files":["a.mts"], "mode":"invalid"}),
            "mode",
        ),
        (
            "reports",
            json!({"type":"dependencies", "files":false}),
            "invalid type",
        ),
        (
            "reports",
            json!({"type":"callSites", "file":false, "exportName":"a"}),
            "invalid type",
        ),
        (
            "reports",
            json!({"type":"effects", "entry":false}),
            "invalid type",
        ),
        (
            "reports",
            json!({"type":"rscCallers", "component":false}),
            "invalid type",
        ),
        (
            "imports",
            json!({"type":"importUsages", "filters":false}),
            "invalid type",
        ),
        (
            "calls",
            json!({"type":"callSites", "file":false, "exportName":"a"}),
            "invalid type",
        ),
    ] {
        let prepared = options(json!([report]));
        let root = Path::new(prepared.root.as_ref().unwrap());
        let session = crate::codebase::analysis_session::AnalysisSession::disabled();
        let error = match helper {
            "symbols" => symbol_target_files(&prepared, root).err(),
            "legacy" => legacy_symbol_target_files(&prepared, root).err(),
            "reports" => authoritative_report_files(&prepared, root).err(),
            "imports" => prepare_import_usage_views(&prepared, root, &session).err(),
            "calls" => prepare_ordinary_call_sites(&prepared, root, session).err(),
            _ => unreachable!(),
        }
        .expect("invalid target must fail");
        assert!(
            format!("{error:#}").contains(expected),
            "{helper}: {error:#}"
        );
    }
}

#[test]
fn authoritative_targets_deduplicate_legacy_symbols_and_omit_absent_optional_paths() {
    let prepared = options(json!([
        {"type":"dependencies", "files":["a.mts#a", {"file":"b.mts", "symbol":"b"}]},
        {"type":"callSites", "file":"a.mts", "exportName":"a"},
        {"type":"effects", "kind":"fetch"},
        {"type":"rscCallers"},
        {"type":"effects", "kind":"fetch", "entry":"c.mts"},
        {"type":"rscCallers", "component":"missing.mts"}
    ]));
    let root = Path::new(prepared.root.as_ref().unwrap());
    let expected = ["a.mts", "b.mts", "c.mts"].map(|file| authoritative_path(root, file.into()));
    assert_eq!(
        authoritative_report_files(&prepared, root).unwrap(),
        expected
    );
    let legacy = legacy_symbol_target_files(&prepared, root).unwrap();
    assert_eq!(
        legacy,
        std::collections::HashSet::from([expected[0].clone()])
    );
}

#[test]
fn ordinary_call_site_targets_share_the_saved_universe_and_skip_unrequested_analysis() {
    let prepared = options(json!([
        {"type":"callSites", "file":"a.mts", "exportName":"a"},
        {"type":"callSites", "file":"a.mts", "exportName":"a"}
    ]));
    let root = Path::new(prepared.root.as_ref().unwrap());
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    let calls = prepare_ordinary_call_sites(&prepared, root, session.clone()).unwrap();
    let files = ordinary_call_site_files(calls.as_ref());
    assert!(files.contains(&authoritative_path(root, "a.mts".into())));
    let absent = prepare_ordinary_call_sites(&options(json!([])), root, session).unwrap();
    assert!(absent.is_none());
    assert!(ordinary_call_site_files(absent.as_ref()).is_empty());
}
