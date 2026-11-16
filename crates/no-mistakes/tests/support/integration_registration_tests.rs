use super::integration_registration::collect;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/integration-suites")
        .join(name)
}

#[test]
fn literal_includes_reach_suite_modules_and_their_root_fragments() {
    let root = fixture("positive");
    let registrations = collect(&root, &[(root.join("suite_entry.rs"), true)]).unwrap();
    let expected = [
        "case.rs",
        "case_fragment.rs",
        "nested/main.rs",
        "suite_entry.rs",
        "suite_parts.rs",
    ]
    .map(|name| (name.to_string(), 1));
    assert_eq!(registrations, BTreeMap::from(expected));
}

#[test]
fn repeated_includes_count_every_registration_without_global_deduplication() {
    let root = fixture("duplicate");
    let registrations = collect(&root, &[(root.join("suite_entry.rs"), true)]).unwrap();
    assert_eq!(
        registrations,
        BTreeMap::from([
            ("case.rs".into(), 2),
            ("case_fragment.rs".into(), 2),
            ("suite_entry.rs".into(), 1),
            ("suite_parts.rs".into(), 2),
        ])
    );
}

#[test]
fn include_cycles_fail_without_recursing_forever() {
    let root = fixture("cycle");
    let error = collect(&root, &[(root.join("suite_entry.rs"), true)]).unwrap_err();
    assert!(error.contains("recursive integration source:"), "{error}");
}

#[test]
fn missing_included_sources_fail_the_inventory() {
    let root = fixture("missing");
    let error = collect(&root, &[(root.join("suite_entry.rs"), true)]).unwrap_err();
    assert!(error.contains("absent.rs"), "{error}");
}

#[test]
fn standalone_targets_keep_their_borrowed_suite_modules_out_of_the_inventory() {
    let root = fixture("positive");
    let registrations = collect(&root, &[(root.join("suite_parts.rs"), false)]).unwrap();
    assert_eq!(
        registrations,
        BTreeMap::from([("suite_parts.rs".into(), 1)])
    );
}
