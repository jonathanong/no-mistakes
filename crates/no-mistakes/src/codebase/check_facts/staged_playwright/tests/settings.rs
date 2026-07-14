use super::*;
use crate::playwright::analysis::context::{DiscoveredTestFile, TestProjectContext};
use crate::playwright::analysis::pipeline_occurrences::{
    prepare_test_files, CachedOccurrenceSelection, PrepareTestFilesOptions,
};
use crate::playwright::playwright_tests::TestStatus;
use crate::playwright::selectors::compile_selector_regexes;

#[test]
fn per_path_variants_preserve_provenance_and_share_common_parse() {
    let path = root().join("tests/multi.spec.ts");
    let mut playwright = PlaywrightFactPlan::default();
    add_variant(
        &mut playwright,
        &path,
        "goA",
        "data-a",
        TestPolicy::default(),
    );
    add_variant(
        &mut playwright,
        &path,
        "goB",
        "data-b",
        TestPolicy {
            assert_conditional_tests: false,
            allow_skipped_tests: true,
        },
    );
    let facts = collect_facts(&[], &[], CheckFactPlan::default(), playwright.clone());
    let cached = facts.ts[&path].playwright.as_ref().unwrap();
    let first_a = prepare(&facts, &path, "goA", "data-a");
    let second_a = prepare(&facts, &path, "goA", "data-a");
    let first_b = prepare(&facts, &path, "goB", "data-b");

    assert_eq!(facts.stats.files_parsed, 1);
    assert_eq!(
        values(first_a.urls()),
        std::collections::BTreeSet::from(["/a"])
    );
    assert_eq!(
        values(first_b.urls()),
        std::collections::BTreeSet::from(["/b"])
    );
    assert_eq!(
        attributes(first_a.selectors()),
        std::collections::BTreeSet::from(["component-a", "data-a"])
    );
    assert_eq!(
        attributes(first_b.selectors()),
        std::collections::BTreeSet::from(["component-b", "data-b"])
    );
    assert!(std::sync::Arc::ptr_eq(&first_a.common, &second_a.common));
    assert!(std::sync::Arc::ptr_eq(&first_a.variant, &second_a.variant));
    assert!(std::sync::Arc::ptr_eq(&first_a.common, &first_b.common));
    assert!(!std::sync::Arc::ptr_eq(&first_a.variant, &first_b.variant));
    assert_eq!(cached.all().len(), 2);
    assert!(playwright.demands_text_imports(&BTreeMap::from([(path.clone(), cached)])));
    assert!(first_a
        .text_locators()
        .iter()
        .any(|locator| locator.status == TestStatus::Skipped));
    assert_missing_variant_is_an_error(&facts, &path);
}

fn assert_missing_variant_is_an_error(facts: &CheckFactMap, path: &std::path::Path) {
    let settings = occurrence_settings(&["goMissing"], &["data-missing"]);
    let regexes = compile_selector_regexes(&settings.selector_attributes, &BTreeMap::new());
    let error = prepare_test_files(
        vec![DiscoveredTestFile {
            path: path.to_path_buf(),
            contexts: vec![TestProjectContext {
                base_url: None,
                test_id_attributes: vec!["data-missing".to_string()],
            }],
        }],
        &settings,
        &regexes,
        PrepareTestFilesOptions {
            test_policy: TestPolicy::default(),
            skip_test_file_errors: false,
            facts: Some(facts),
            selection: CachedOccurrenceSelection::Exact,
            module_resolution: None,
        },
    )
    .err()
    .unwrap();

    assert!(error.to_string().contains("lack the requested variant"));
}

fn add_variant(
    plan: &mut PlaywrightFactPlan,
    path: &std::path::Path,
    helper: &str,
    attribute: &str,
    policy: TestPolicy,
) {
    let suffix = attribute.trim_start_matches("data-");
    let component_attributes =
        BTreeMap::from([(format!("component{suffix}"), format!("component-{suffix}"))]);
    plan.add_file(PlaywrightFactSelection {
        path: path.to_path_buf(),
        navigation_helpers: &[helper.to_string()],
        selector_wrappers: &[],
        selector_attributes: &[attribute.to_string()],
        component_selector_attributes: &component_attributes,
        html_ids: false,
        test_id_attributes: &[attribute.to_string()],
        policy,
        demands_text_imports: true,
    });
}

fn prepare(
    facts: &CheckFactMap,
    path: &std::path::Path,
    helper: &str,
    attribute: &str,
) -> crate::playwright::test_file_occurrences::TestFileOccurrences {
    let suffix = attribute.trim_start_matches("data-");
    let mut settings = occurrence_settings(&[helper], &[attribute]);
    settings.component_selector_attributes =
        BTreeMap::from([(format!("component{suffix}"), format!("component-{suffix}"))]);
    let regexes = compile_selector_regexes(
        &settings.selector_attributes,
        &settings.component_selector_attributes,
    );
    prepare_test_files(
        vec![DiscoveredTestFile {
            path: path.to_path_buf(),
            contexts: vec![TestProjectContext {
                base_url: None,
                test_id_attributes: vec![attribute.to_string()],
            }],
        }],
        &settings,
        &regexes,
        PrepareTestFilesOptions {
            test_policy: TestPolicy::default(),
            skip_test_file_errors: false,
            facts: Some(facts),
            selection: CachedOccurrenceSelection::Exact,
            module_resolution: None,
        },
    )
    .unwrap()
    .0
    .pop()
    .unwrap()
    .occurrences
}

fn values(
    occurrences: &[crate::playwright::playwright_tests::TestOccurrence<String>],
) -> std::collections::BTreeSet<&str> {
    occurrences
        .iter()
        .map(|occurrence| occurrence.value.as_str())
        .collect()
}

fn attributes(
    occurrences: &[crate::playwright::playwright_tests::TestOccurrence<
        crate::playwright::selectors::PlaywrightSelector,
    >],
) -> std::collections::BTreeSet<&str> {
    occurrences
        .iter()
        .map(|occurrence| occurrence.value.attribute.as_str())
        .collect()
}
