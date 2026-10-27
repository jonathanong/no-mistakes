use super::*;

// Output cases moved to cli_postgres_snapshot_generated_column_writes.

const COLUMNS: &str = "triggerMaintainedColumns: [updated_at]\n";

fn root() -> PathBuf {
    unit_fixture("trigger-maintained")
}

fn files(names: &[&str]) -> Vec<PathBuf> {
    let root = root();
    names.iter().map(|name| root.join(name)).collect()
}

fn messages(yaml: &str, names: &[&str]) -> Vec<String> {
    check_with_files(&root(), &config_with_options(yaml), &files(names))
        .unwrap()
        .into_iter()
        .map(|finding| finding.message)
        .collect()
}

#[test]
fn config_errors_name_the_option() {
    for (yaml, expected) in [
        (
            "triggerMaintainedColumns: ['']\n",
            "option triggerMaintainedColumns: empty column name",
        ),
        (
            "triggerMaintainedColumns: [updated_at, UPDATED_AT]\n",
            "option triggerMaintainedColumns: duplicate entry updated_at",
        ),
    ] {
        let error = check_with_files(&root(), &config_with_options(yaml), &files(&["schema.sql"]))
            .expect_err("config")
            .to_string();
        assert!(error.contains(expected), "{error}");
    }
}

#[test]
fn suppression_directives_hide_trigger_writes() {
    let paths = files(&[
        "schema.sql",
        "suppress-next.ts",
        "suppress-line.ts",
        "suppress-file.ts",
    ]);
    let mut findings = check_with_files(&root(), &config_with_options(COLUMNS), &paths).unwrap();
    assert_eq!(findings.len(), 3, "{findings:?}");
    let sources = crate::codebase::rules::source_store_for_files(&paths);
    crate::codebase::rules::suppress_rule_findings_with_sources(&root(), &mut findings, &sources);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn repeated_scans_match() {
    let names = ["schema.sql", "fail-update.ts"];
    assert_eq!(messages(COLUMNS, &names), messages(COLUMNS, &names));
}
