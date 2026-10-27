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

fn pipeline_error_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/codebase/symbols-pipeline-errors")
        .join(name)
}

#[test]
fn collect_entries_reports_an_explicit_missing_tsconfig() {
    let root = pipeline_error_fixture("explicit-missing-tsconfig");
    let mut args = args_for(&root, vec!["src.ts"], Format::Json);
    args.tsconfig = Some(root.join("missing-tsconfig.json"));
    let err = collect_entries(&args).unwrap_err();
    assert!(!format!("{err:#}").is_empty());
}

#[test]
fn collect_entries_falls_back_when_the_default_tsconfig_is_invalid() {
    let root = pipeline_error_fixture("invalid-default-tsconfig");
    let args = args_for(&root, vec!["src.ts"], Format::Json);
    let (entries, _) = collect_entries(&args).expect("invalid default tsconfig falls back");
    assert_eq!(entries.len(), 1);
}

#[test]
fn collect_entries_accepts_a_relative_tsconfig_path() {
    let root = pipeline_error_fixture("relative-tsconfig");
    let mut args = args_for(&root, vec!["src.ts"], Format::Json);
    args.tsconfig = Some(PathBuf::from("tsconfig.json"));
    let (entries, _) = collect_entries(&args).expect("relative tsconfig is joined to root");
    assert_eq!(entries.len(), 1);
}
