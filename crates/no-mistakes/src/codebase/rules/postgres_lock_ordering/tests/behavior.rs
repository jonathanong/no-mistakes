use super::*;

#[test]
fn fail_fixture_reports_abba_deadlock() {
    let findings = findings_for("fail");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].rule, RULE_ID);
    assert_eq!(findings[0].file, "src/lock.ts");
    assert!(findings[0].line > 0);
    assert_eq!(findings[0].target.as_deref(), Some(LOCK_ORDERING_TARGET));
    assert!(findings[0].message.contains("ABBA"), "{findings:#?}");
    assert!(
        findings[0].message.contains("deadlock-safe"),
        "{findings:#?}"
    );
}

#[test]
fn order_by_is_safe() {
    assert!(findings_for("pass-order").is_empty());
}

#[test]
fn catalog_mode_requires_a_real_unique_key_prefix() {
    assert!(findings_with_catalog("pass-catalog").is_empty());
    assert!(findings_with_catalog("pass-catalog-of-alias").is_empty());
    assert!(findings_with_catalog("pass-catalog-unqualified").is_empty());
    let findings = findings_with_catalog("fail-catalog-order");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert!(findings[0].message.contains("schema-catalog"));
    let findings = findings_with_catalog("fail-catalog-join");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert!(findings[0].message.contains("schema-catalog"));
}

#[test]
fn catalog_mode_scopes_qualifiers_and_schema_names_to_locked_relations() {
    for scenario in [
        "fail-catalog-qualified-other",
        "fail-catalog-schema-qualified",
    ] {
        let findings = findings_with_catalog(scenario);
        assert_eq!(findings.len(), 1, "{scenario}: {findings:#?}");
        assert!(findings[0].message.contains("schema-catalog"));
    }
}

#[test]
fn catalog_mode_treats_pinned_leading_key_columns_as_ordered() {
    assert!(findings_with_catalog("pass-pinned-key-prefix").is_empty());
    let findings = findings_with_catalog("fail-pinned-key-prefix");
    let lines: Vec<_> = findings.iter().map(|finding| finding.line).collect();
    assert_eq!(lines, [5, 10, 15, 20, 25, 30, 35], "{findings:#?}");
    for finding in &findings {
        assert!(
            finding.message.contains("schema-catalog unique-key order"),
            "{finding:#?}"
        );
        assert_eq!(finding.target.as_deref(), Some(LOCK_ORDERING_TARGET));
    }
}

#[test]
fn skip_locked_is_safe() {
    assert!(findings_for("pass-skip").is_empty());
}

#[test]
fn deadlock_safe_directive_suppresses() {
    assert!(findings_for("pass-directive").is_empty());
}

#[test]
fn unparseable_sql_has_distinct_diagnostic() {
    let findings = findings_for("unparseable");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].target.as_deref(), Some(UNPARSEABLE_TARGET));
    assert!(findings[0].message.contains("parseable"), "{findings:#?}");
}

#[test]
fn honors_disable_comments() {
    let root = fixture("fail");
    let file = root.join("src/disabled.ts");
    let mut findings =
        check_with_files(&root, &default_config(), std::slice::from_ref(&file)).unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    let sources = super::super::super::source_store_for_files(std::slice::from_ref(&file));
    super::super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn include_and_exclude_globs_filter_files() {
    let root = fixture("fail");
    let files = vec![ts_file(&root)];
    let excluded = check_with_files(
        &root,
        &config_with_options("exclude: ['src/lock.ts']"),
        &files,
    )
    .unwrap();
    assert!(excluded.is_empty(), "{excluded:#?}");

    let included = check_with_files(
        &root,
        &config_with_options("include: ['src/lock.ts']"),
        &files,
    )
    .unwrap();
    assert_eq!(included.len(), 1);

    let missed = check_with_files(
        &root,
        &config_with_options("include: ['src/missing.ts']"),
        &files,
    )
    .unwrap();
    assert!(missed.is_empty(), "{missed:#?}");
}
