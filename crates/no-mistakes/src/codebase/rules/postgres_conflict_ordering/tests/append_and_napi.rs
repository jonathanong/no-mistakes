#[test]
fn allows_sql_template_strings_append_mutations() {
    let root = fixture("pass-append-mutations");
    let result = check_with_files(
        &root,
        &config(),
        &[
            root.join("src/select.ts"),
            root.join("src/update.ts"),
            root.join("src/conditional.ts"),
            root.join("src/helper.ts"),
            root.join("src/helper-update.ts"),
            root.join("src/with-update.ts"),
            root.join("src/with-insert.ts"),
            root.join("src/insert.ts"),
            root.join("schema.json"),
        ],
    )
    .unwrap();
    assert!(result.is_empty(), "{result:#?}");
}

#[test]
fn rejects_recovered_append_insert_without_canonical_order() {
    let findings = findings("fail-append-missing-order");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(
        findings[0].target.as_deref(),
        Some("missing-canonical-order")
    );
}

#[test]
fn checks_both_conditional_append_paths_independently() {
    let findings = findings("fail-append-conditional-insert");
    // The taken path has canonical conflict ordering; the untaken path has
    // no ON CONFLICT, so conflict-ordering does not apply to that version.
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn rejects_opaque_append_insert_as_unanalyzable() {
    let findings = findings("fail-append-opaque-insert");
    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert!(
        findings
            .iter()
            .all(|finding| finding.target.as_deref() == Some("unanalyzable-sql")),
        "{findings:#?}"
    );
}

#[test]
fn rejects_incomplete_with_plus_opaque_append_as_unanalyzable() {
    let findings = findings("fail-incomplete-with-append");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].target.as_deref(), Some("unanalyzable-sql"));
}

#[test]
fn rejects_recovered_helper_insert_without_canonical_order() {
    let findings = findings("fail-helper-insert-missing-order");
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(
        findings[0].target.as_deref(),
        Some("missing-canonical-order")
    );
}

#[test]
fn napi_check_reports_the_same_registered_rule() {
    let root = fixture("fail-missing-order");
    let report = crate::napi_api::check_json_impl(crate::napi_api::options::test_json_arg(
        serde_json::json!({ "root": root, "config": root.join(".no-mistakes.yml") }).to_string(),
    ))
    .unwrap();
    let report: serde_json::Value = serde_json::from_str(&report).unwrap();
    assert!(
        report["rules"].as_array().is_some_and(|findings| {
            findings.iter().any(|finding| {
                finding["rule"] == RULE_ID && finding["target"] == "missing-canonical-order"
            })
        }),
        "{report:#?}"
    );
}
