use super::*;

#[test]
fn file_disabled_parse_errors_are_skipped_before_react_aggregation() {
    let root = fixture("nested");
    let file = root.join("app/components/Child.tsx");
    let mut shared = facts(Vec::new());
    shared.files.push(file.clone());
    shared.ts.insert(
        file,
        CheckFileFacts {
            parse_error: Some("synthetic disabled parse error".to_string()),
            source: Some("// no-mistakes-disable-file assert-no-fetch: generated source\n<".into()),
            ..Default::default()
        }
        .into(),
    );

    let findings = run_analyze_inner_with_facts(
        &root,
        &FileConfig {
            frontend_root: Some("app".to_string()),
            assert_no_fetch: Some(true),
        },
        &["app/components/Child.tsx".to_string()],
        &shared,
    )
    .unwrap();

    assert!(findings.is_empty());
}
