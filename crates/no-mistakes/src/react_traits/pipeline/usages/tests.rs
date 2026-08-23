use super::*;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/react-traits-usages/basic/fixture")
}

#[test]
fn usages_with_symbol_reports_callsites_stories_tests_and_prop_types() {
    let root = fixture();
    let report = run_usages(
        &root,
        None,
        "app/components/button.tsx#Button",
        &[],
        &UsagesInclude::all(),
    )
    .unwrap();

    assert_eq!(report.target.file, "app/components/button.tsx");
    assert_eq!(report.target.symbol.as_deref(), Some("Button"));
    assert_eq!(report.callsites.len(), 5);
    // Sorted by (file, line).
    let files: Vec<&str> = report.callsites.iter().map(|c| c.file.as_str()).collect();
    let mut sorted = files.clone();
    sorted.sort();
    assert_eq!(files, sorted);

    let home = report
        .callsites
        .iter()
        .find(|c| c.file == "app/pages/home.tsx")
        .unwrap();
    assert_eq!(home.component, "Button");
    assert_eq!(
        home.props,
        vec!["variant".to_string(), "onClick".to_string()]
    );
    assert!(!home.has_spread);

    let dashboard = report
        .callsites
        .iter()
        .find(|c| c.file == "app/pages/dashboard.tsx")
        .unwrap();
    assert!(dashboard.has_spread);
    assert!(dashboard.props.is_empty());

    assert_eq!(
        report.stories.unwrap(),
        vec!["app/components/button.stories.tsx".to_string()]
    );
    assert_eq!(
        report.tests.unwrap(),
        vec!["app/components/button.test.tsx".to_string()]
    );
    assert_eq!(
        report.prop_types.unwrap(),
        vec!["ButtonProps".to_string(), "ButtonVariant".to_string()]
    );
}

#[test]
fn usages_without_symbol_matches_all_exports_in_file() {
    let root = fixture();
    let report = run_usages(
        &root,
        None,
        "app/components/button.tsx",
        &[],
        &UsagesInclude::all(),
    )
    .unwrap();
    assert!(report.target.symbol.is_none());
    assert_eq!(report.callsites.len(), 5);
}

#[test]
fn include_props_only_omits_stories_and_tests() {
    let root = fixture();
    let include = UsagesInclude::parse(Some("props")).unwrap();
    let report = run_usages(
        &root,
        None,
        "app/components/button.tsx#Button",
        &[],
        &include,
    )
    .unwrap();
    assert!(report.stories.is_none());
    assert!(report.tests.is_none());
    assert!(report.prop_types.is_some());
    assert_eq!(report.callsites.len(), 5);
}

#[test]
fn unknown_symbol_yields_no_callsites_but_keeps_prop_types() {
    let root = fixture();
    let report = run_usages(
        &root,
        None,
        "app/components/button.tsx#DoesNotExist",
        &[],
        &UsagesInclude::all(),
    )
    .unwrap();
    assert!(report.callsites.is_empty());
    assert!(report.stories.unwrap().is_empty());
    assert!(report.tests.unwrap().is_empty());
    assert!(!report.prop_types.unwrap().is_empty());
}

#[test]
fn scan_targets_narrow_the_search() {
    let root = fixture();
    let report = run_usages(
        &root,
        None,
        "app/components/button.tsx#Button",
        &["app/pages/**/*.tsx".to_string()],
        &UsagesInclude::all(),
    )
    .unwrap();
    assert_eq!(report.callsites.len(), 3);
    assert!(report.stories.unwrap().is_empty());
}

#[test]
fn absolute_target_path_is_accepted() {
    let root = fixture();
    let absolute = root.join("app/components/button.tsx");
    let target = format!("{}#Button", absolute.to_str().unwrap());
    let report = run_usages(&root, None, &target, &[], &UsagesInclude::all()).unwrap();
    assert_eq!(report.callsites.len(), 5);
}

#[test]
fn target_file_not_found_is_an_error() {
    let root = fixture();
    let err = run_usages(
        &root,
        None,
        "app/components/missing.tsx",
        &[],
        &UsagesInclude::all(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("target file not found"));
}

#[test]
fn loaded_usages_rejects_an_absolute_missing_target() {
    let root = fixture();
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let missing = root.join("app/components/missing.tsx");
    let error = run_usages_with_loaded_config_and_facts(
        &root,
        &config,
        &missing.to_string_lossy(),
        &[],
        &UsagesInclude::all(),
        &crate::codebase::check_facts::CheckFactMap::default(),
    )
    .unwrap_err();
    assert!(error.to_string().contains("target file not found"));
}

#[test]
fn loaded_usages_skips_files_without_prepared_usage_facts() {
    let root = fixture();
    let target = root.join("app/components/button.tsx");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let facts = crate::codebase::check_facts::CheckFactMap {
        files: vec![target.clone()],
        ..Default::default()
    };
    let report = run_usages_with_loaded_config_and_facts(
        &root,
        &config,
        "app/components/button.tsx#Button",
        &[],
        &UsagesInclude::all(),
        &facts,
    )
    .unwrap();
    assert!(report.callsites.is_empty());
    assert!(report.stories.unwrap().is_empty());
    assert!(report.tests.unwrap().is_empty());
}

#[test]
fn run_usages_returns_error_for_nonexistent_config_path() {
    let root = fixture();
    let error = run_usages(
        &root,
        Some(Path::new("missing-no-mistakes.yaml")),
        "app/components/button.tsx#Button",
        &[],
        &UsagesInclude::all(),
    )
    .unwrap_err();
    assert!(error.to_string().contains("missing-no-mistakes.yaml"));
}

#[test]
fn collect_usage_file_facts_supports_unscoped_import_resolution() {
    let file = fixture().join("app/pages/home.tsx");
    let source = std::fs::read_to_string(&file).unwrap();
    let facts = crate::ast::with_program(&file, &source, |program, _| {
        collect_usage_file_facts(&file, &source, program, None)
    })
    .unwrap();
    assert!(!facts.imports.is_empty());
    assert!(!facts.callsites.is_empty());
}

#[test]
fn missing_target_file_yields_empty_prop_types() {
    let missing = fixture().join("app/components/not-on-disk.tsx");
    assert!(super::helpers::prop_type_names_with_sources(&missing, None).is_empty());
}

#[test]
fn unparseable_target_file_yields_empty_prop_types() {
    let root = fixture();
    // broken.tsx is unparseable: scanning skips it, and its prop types come back empty.
    let report = run_usages(
        &root,
        None,
        "app/junk/broken.tsx#Broken",
        &[],
        &UsagesInclude::all(),
    )
    .unwrap();
    assert!(report.callsites.is_empty());
    assert!(report.prop_types.unwrap().is_empty());
}

#[test]
fn include_parse_handles_subsets_and_rejects_unknown() {
    let all = UsagesInclude::parse(None).unwrap();
    assert!(all.stories && all.tests && all.prop_types);
    let subset = UsagesInclude::parse(Some("stories, tests")).unwrap();
    assert!(subset.stories && subset.tests && !subset.prop_types);
    let empty = UsagesInclude::parse(Some("")).unwrap();
    assert!(!empty.stories && !empty.tests && !empty.prop_types);
    assert!(UsagesInclude::parse(Some("bogus")).is_err());
}

#[test]
fn split_target_parses_symbol_suffix() {
    assert_eq!(
        split_target("a/b.tsx#Btn"),
        ("a/b.tsx", Some("Btn".to_string()))
    );
    assert_eq!(split_target("a/b.tsx#"), ("a/b.tsx", None));
    assert_eq!(split_target("a/b.tsx"), ("a/b.tsx", None));
}

#[test]
fn classifies_story_and_test_files() {
    // Nested and bare paths exercise both basename branches.
    assert!(is_story("app/components/x.stories.tsx"));
    assert!(is_story("x.stories.tsx"));
    assert!(!is_story("app/x.tsx"));
    assert!(is_test("a/b/c.test.ts"));
    assert!(is_test("a.spec.tsx"));
    assert!(!is_test("a.tsx"));
}
