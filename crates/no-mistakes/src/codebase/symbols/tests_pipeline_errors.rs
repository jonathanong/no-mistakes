#[test]
fn collect_entries_reports_unreadable_input_files() {
    let args = fixture_args(vec!["src/does-not-exist.mts"], Format::Json);
    let err = collect_entries(&args).unwrap_err();
    assert!(format!("{err:#}").contains("reading"));
}

#[test]
fn collect_entries_with_prepared_facts_reports_parse_errors() {
    use crate::codebase::check_facts::CheckFileFacts;
    use crate::codebase::ts_source::facts::TsFileFacts;
    use std::sync::Arc;

    let root = fixture_root();
    let args = fixture_args(vec!["src/utils.mts"], Format::Json);
    let path = crate::codebase::ts_resolver::normalize_path(&root.join("src/utils.mts"));
    let session = crate::codebase::analysis_session::AnalysisSession::new(None);
    let visible: crate::fx::PathSet = std::iter::empty::<PathBuf>().collect();
    let catalog = crate::codebase::ts_resolver::TsConfigCatalog::from_visible(
        &root,
        std::slice::from_ref(&root),
        &[],
    );
    let mut facts = crate::codebase::check_facts::CheckFactMap::default();
    facts.ts.insert(
        path,
        Arc::new(CheckFileFacts {
            ts: Arc::new(TsFileFacts {
                parse_error: Some("boom".to_string()),
                ..TsFileFacts::default()
            }),
            parse_error: Some("boom".to_string()),
            ..CheckFileFacts::default()
        }),
    );
    let err = collect_entries_with_prepared_facts(
        &args,
        &root,
        &catalog,
        &visible,
        &facts,
        &facts,
        session.as_ref(),
    )
    .unwrap_err();
    assert!(format!("{err:#}").contains("extracting symbols"));
}

#[test]
fn collect_entries_with_prepared_facts_reports_missing_symbols() {
    use crate::codebase::check_facts::CheckFileFacts;
    use crate::codebase::ts_source::facts::TsFileFacts;
    use std::sync::Arc;

    let root = fixture_root();
    let args = fixture_args(vec!["src/utils.mts"], Format::Json);
    let path = crate::codebase::ts_resolver::normalize_path(&root.join("src/utils.mts"));
    let session = crate::codebase::analysis_session::AnalysisSession::new(None);
    let visible: crate::fx::PathSet = std::iter::empty::<PathBuf>().collect();
    let catalog = crate::codebase::ts_resolver::TsConfigCatalog::from_visible(
        &root,
        std::slice::from_ref(&root),
        &[],
    );
    let mut facts = crate::codebase::check_facts::CheckFactMap::default();
    facts.ts.insert(
        path,
        Arc::new(CheckFileFacts {
            ts: Arc::new(TsFileFacts::default()),
            ..CheckFileFacts::default()
        }),
    );
    let err = collect_entries_with_prepared_facts(
        &args,
        &root,
        &catalog,
        &visible,
        &facts,
        &facts,
        session.as_ref(),
    )
    .unwrap_err();
    assert!(format!("{err:#}").contains("missing symbols"));
}
