use super::*;

#[test]
fn inventory_rejects_invalid_names_duplicate_environment_scopes_and_unknown_options() {
    for (file, message) in [
        (
            "invalid-empty.yml",
            "invalid secretInventory name in repository",
        ),
        (
            "invalid-first.yml",
            "invalid secretInventory name in repository",
        ),
        (
            "invalid-char.yml",
            "invalid secretInventory name in repository",
        ),
        (
            "environment-empty.yml",
            "environment names must not be empty",
        ),
        (
            "environment-duplicate.yml",
            "duplicate secretInventory environment",
        ),
        (
            "environment-secret-duplicate.yml",
            "duplicate secretInventory name in Prod",
        ),
        (
            "builtin-duplicate.yml",
            "duplicate secretInventory name in builtInExemptions",
        ),
        (
            "builtin-invalid.yml",
            "invalid secretInventory name in builtInExemptions",
        ),
        ("unknown-option.yml", "unknown field `unknownKey`"),
    ] {
        let error = run("validation", file).unwrap_err().to_string();
        assert!(error.contains(message), "{file}: {error}");
    }
}

#[test]
fn public_filesystem_dispatch_applies_file_line_and_next_line_suppressions_to_real_locations() {
    let root = fixture("suppression");
    let findings = crate::codebase::rules::filesystem_dispatch::run_filesystem_rules(
        &root,
        Some(&root.join(".no-mistakes.yml")),
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].rule, RULE_ID);
    assert_eq!(findings[0].file, ".github/workflows/check.yml");
    assert_eq!(findings[0].line, 9);
    assert!(findings[0].message.contains("undeclared secret THREE"));
}
