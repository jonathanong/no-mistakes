use super::*;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/data-pw/fixture")
}

#[test]
fn include_parse_default_and_subsets() {
    assert_eq!(
        DataPwInclude::parse(None).unwrap(),
        DataPwInclude::default()
    );
    assert_eq!(
        DataPwInclude::parse(Some("source")).unwrap(),
        DataPwInclude {
            source: true,
            test: false
        }
    );
    assert_eq!(
        DataPwInclude::parse(Some("test")).unwrap(),
        DataPwInclude {
            source: false,
            test: true
        }
    );
    // empty segments are ignored
    assert_eq!(
        DataPwInclude::parse(Some("source, ,test")).unwrap(),
        DataPwInclude::default()
    );
}

#[test]
fn include_parse_rejects_unknown_and_empty() {
    assert!(DataPwInclude::parse(Some("bogus")).is_err());
    assert!(DataPwInclude::parse(Some(" , ")).is_err());
}

#[test]
fn path_in_root_matches_directory_prefix_only() {
    assert!(path_in_root("app/page.tsx", "app"));
    assert!(path_in_root("app", "app"));
    assert!(!path_in_root("apply.ts", "app"));
    assert!(!path_in_root("components/widget.tsx", "app"));
}

#[test]
fn report_paths_unions_sections() {
    let report = DataPwReport {
        value: "x".into(),
        attributes: vec!["data-pw".into()],
        source: Some(vec![DataPwHit {
            file: "app/a.tsx".into(),
            line: 1,
            attribute: "data-pw".into(),
        }]),
        test: Some(vec![DataPwHit {
            file: "e2e/a.spec.ts".into(),
            line: 2,
            attribute: "data-pw".into(),
        }]),
    };
    assert_eq!(report.paths(), vec!["app/a.tsx", "e2e/a.spec.ts"]);
}

#[test]
fn finds_source_and_test_usages() {
    let report = run(
        &fixture(),
        None,
        "search-bar",
        &[],
        &[],
        &DataPwInclude::default(),
    )
    .unwrap();
    let source = report.source.unwrap();
    // app/search.tsx (data-pw) + components/widget.tsx (data-testid); the
    // dynamic value, the near-miss attribute, the excluded file, and the
    // out-of-root file are all absent.
    let source_files: Vec<&str> = source.iter().map(|h| h.file.as_str()).collect();
    assert_eq!(
        source_files,
        vec!["app/search.tsx", "components/widget.tsx"]
    );
    assert_eq!(source[0].line, 3);
    assert_eq!(source[0].attribute, "data-pw");

    let test = report.test.unwrap();
    assert_eq!(test.len(), 1);
    assert_eq!(test[0].file, "e2e/search.spec.ts");
    assert_eq!(test[0].attribute, "data-pw");
}

#[test]
fn attribute_override_restricts_scan() {
    let report = run(
        &fixture(),
        None,
        "search-bar",
        &["data-testid".to_string()],
        &[],
        &DataPwInclude::default(),
    )
    .unwrap();
    let source = report.source.unwrap();
    let files: Vec<&str> = source.iter().map(|h| h.file.as_str()).collect();
    assert_eq!(files, vec!["components/widget.tsx"]);
    assert!(report.test.unwrap().is_empty());
}

#[test]
fn scan_override_changes_source_roots() {
    let report = run(
        &fixture(),
        None,
        "search-bar",
        &[],
        &["other".to_string()],
        &DataPwInclude::default(),
    )
    .unwrap();
    let source = report.source.unwrap();
    let files: Vec<&str> = source.iter().map(|h| h.file.as_str()).collect();
    assert_eq!(files, vec!["other/elsewhere.tsx"]);
}

#[test]
fn include_filters_sections() {
    let report = run(
        &fixture(),
        None,
        "search-bar",
        &[],
        &[],
        &DataPwInclude {
            source: false,
            test: true,
        },
    )
    .unwrap();
    assert!(report.source.is_none());
    assert!(report.test.is_some());
}

#[test]
fn value_not_found_is_empty() {
    let report = run(
        &fixture(),
        None,
        "nope",
        &[],
        &[],
        &DataPwInclude::default(),
    )
    .unwrap();
    assert!(report.source.unwrap().is_empty());
    assert!(report.test.unwrap().is_empty());
}

#[test]
fn dot_scan_root_scans_whole_repo() {
    // `.` means "scan everything", so out-of-default-root source is included.
    let report = run(
        &fixture(),
        None,
        "search-bar",
        &[],
        &[".".to_string()],
        &DataPwInclude::default(),
    )
    .unwrap();
    let source = report.source.unwrap();
    let files: Vec<&str> = source.iter().map(|h| h.file.as_str()).collect();
    assert!(files.contains(&"other/elsewhere.tsx"));
    assert!(files.contains(&"app/search.tsx"));
}

#[test]
fn honors_test_exclude_and_selector_include() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/data-pw-globs/fixture");
    let report = run(&root, None, "x", &[], &[], &DataPwInclude::default()).unwrap();
    let source_hits = report.source.unwrap();
    let source: Vec<&str> = source_hits.iter().map(|h| h.file.as_str()).collect();
    // selectorInclude keeps src/keep, drops src/skip.
    assert_eq!(source, vec!["src/keep/widget.tsx"]);
    let test_hits = report.test.unwrap();
    let test: Vec<&str> = test_hits.iter().map(|h| h.file.as_str()).collect();
    // testExclude drops the flaky spec.
    assert_eq!(test, vec!["e2e/main.spec.ts"]);
}

#[test]
fn is_skip_dir_honors_defaults_and_config() {
    assert!(is_skip_dir(Path::new("x/node_modules"), &[]));
    assert!(is_skip_dir(Path::new("x/.cache"), &[]));
    assert!(is_skip_dir(
        Path::new("x/generated"),
        &["generated".to_string()]
    ));
    assert!(!is_skip_dir(Path::new("x/app"), &[]));
}

#[test]
fn scan_file_ignores_unreadable_path() {
    let regex = compile_selector_attribute_value_regex(&["data-pw".to_string()]).unwrap();
    let globs = build_globset(&[]).unwrap();
    let scan = ScanConfig {
        value: "v",
        regex: &regex,
        roots: &[],
        test_globs: &globs,
        test_exclude_globs: &globs,
        selector_include_globs: None,
        exclude_globs: &globs,
    };
    let hits = scan_file(Path::new("/no/such/file.tsx"), "x.tsx", &scan);
    assert!(hits.is_empty());
}

#[test]
fn errors_without_configured_attributes() {
    // A directory with no config and no --attribute override has no testIds.
    let tmp = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases");
    let err = run(&tmp, None, "x", &[], &[], &DataPwInclude::default()).unwrap_err();
    assert!(err
        .to_string()
        .contains("no selector attributes configured"));
}
