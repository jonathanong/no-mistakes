use super::*;

// Output cases moved to cli_gha_snapshot_workflow_topology.

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
