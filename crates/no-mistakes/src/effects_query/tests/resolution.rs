use super::*;

fn resolved_fixture() -> tempfile::TempDir {
    crate::test_support::materialize_saved_fixture(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/queries/resolved-effects"),
    )
}

#[test]
fn module_export_targets_match_aliases_and_namespaces_without_matching_local_bindings() {
    let fixture = resolved_fixture();
    let report = run(
        fixture.path(),
        None,
        None,
        "resolved",
        Path::new("entry.ts"),
        &[],
        None,
    )
    .unwrap();
    assert_eq!(
        report
            .call_sites
            .iter()
            .map(|site| site.line)
            .collect::<Vec<_>>(),
        vec![3, 4, 6, 8, 9, 16, 18]
    );
    assert_eq!(report.by_category.get("storage"), Some(&6));
    assert_eq!(report.by_category.get("uncategorized"), Some(&1));
    assert_eq!(
        report
            .call_sites
            .iter()
            .find(|site| site.line == 16)
            .unwrap()
            .caller
            .as_deref(),
        Some("execute")
    );
    let filtered = run(
        fixture.path(),
        None,
        None,
        "resolved",
        Path::new("entry.ts"),
        &["storage".into()],
        None,
    )
    .unwrap();
    assert_eq!(filtered.call_sites.len(), 6);
}

#[test]
fn legacy_names_remain_compatible_and_duplicate_targets_do_not_duplicate_occurrences() {
    let fixture = resolved_fixture();
    let report = run(
        fixture.path(),
        None,
        None,
        "mixed",
        Path::new("entry.ts"),
        &[],
        None,
    )
    .unwrap();
    assert_eq!(
        report
            .call_sites
            .iter()
            .map(|site| site.line)
            .collect::<Vec<_>>(),
        vec![3, 4, 6, 8, 11, 13, 16, 18]
    );
    assert_eq!(
        report
            .call_sites
            .iter()
            .filter(|site| site.line == 4)
            .count(),
        1
    );
    assert_eq!(
        report
            .call_sites
            .iter()
            .find(|site| site.line == 4)
            .unwrap()
            .category,
        None
    );
}

#[test]
fn repository_effect_targets_retain_the_direct_import_specifier_boundary() {
    let fixture = resolved_fixture();
    let report = run(
        fixture.path(),
        None,
        None,
        "repository",
        Path::new("entry.ts"),
        &[],
        None,
    )
    .unwrap();
    assert_eq!(
        report
            .call_sites
            .iter()
            .map(|site| site.line)
            .collect::<Vec<_>>(),
        vec![20]
    );
}

#[test]
fn module_export_targets_validate_empty_selectors_and_unknown_fields() {
    let fixture = resolved_fixture();
    let error = run(
        fixture.path(),
        None,
        None,
        "invalid",
        Path::new("entry.ts"),
        &[],
        None,
    )
    .unwrap_err();
    assert!(error.to_string().contains("non-empty module and export"));
    assert!(
        serde_yaml::from_str::<crate::config::v2::schema::EffectTargetConfig>(
            "module: pkg\nexport: write\nunknown: true"
        )
        .is_err()
    );
}
