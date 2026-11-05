use super::*;

#[test]
fn file_disabled_parse_errors_remain_prerequisite_failures() {
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

    let analysis_error = run_analyze_inner_with_facts(
        &root,
        &FileConfig {
            frontend_root: Some("app".to_string()),
            assert_no_fetch: Some(true),
        },
        &["app/components/Child.tsx".to_string()],
        &shared,
    )
    .unwrap_err();
    assert!(analysis_error
        .to_string()
        .contains("synthetic disabled parse error"));

    let error = run_with_session(
        &root,
        &FileConfig {
            frontend_root: Some("app".to_string()),
            assert_no_fetch: Some(true),
        },
        &["app/components/Child.tsx".to_string()],
        &shared,
        None,
    )
    .err()
    .unwrap();
    assert!(error.to_string().contains("synthetic disabled parse error"));
}

#[test]
fn failed_used_child_marks_parent_incomplete_and_keeps_independent_component() {
    let root = fixture("partial-prerequisites");
    let files = crate::codebase::ts_source::discover_visible_paths(&root);
    let shared = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            react: true,
            ..Default::default()
        },
    );
    assert!(shared
        .ts
        .iter()
        .any(|(path, facts)| path.ends_with("Child.tsx") && facts.parse_error.is_some()));
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    session.enable_partial_checks();
    let file_config = FileConfig {
        frontend_root: Some("app".into()),
        assert_no_fetch: Some(true),
    };
    let results = run_with_session(
        &root,
        &file_config,
        &[
            "app/components/Parent.tsx".into(),
            "app/components/Independent.tsx".into(),
        ],
        &shared,
        Some(&session),
    )
    .unwrap();
    assert_eq!(results.len(), 1);
    assert!(!results[0].facts.fetches.is_empty());
    assert_eq!(results[0].facts.name, "Independent");
    assert_eq!(session.check_failures().len(), 1);
    assert!(run_analyze_inner_with_facts(
        &root,
        &file_config,
        &["app/components/Parent.tsx".into()],
        &shared
    )
    .is_err());
    let unaffected = run_analyze_inner_with_facts(
        &root,
        &file_config,
        &["app/components/Independent.tsx".into()],
        &shared,
    )
    .unwrap();
    assert_eq!(unaffected.len(), 1);
}
