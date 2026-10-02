use super::*;

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
fn invalid_writes_are_reported() {
    let body = messages(
        COLUMNS,
        &[
            "schema.sql",
            "fail-update.ts",
            "fail-clock.ts",
            "fail-insert.ts",
            "fail-upsert.ts",
            "fail-noop.ts",
            "fail-merge.sql",
        ],
    )
    .join("\n");
    assert!(
        body.contains("do not write trigger-maintained column `orders.updated_at`"),
        "{body}"
    );
    assert!(body.contains("fail-update.ts"), "{body}");
    assert!(body.contains("fail-clock.ts"), "{body}");
    assert!(body.contains("fail-insert.ts"), "{body}");
    assert!(body.contains("fail-upsert.ts"), "{body}");
    assert!(body.contains("fail-noop.ts"), "{body}");
    assert!(body.contains("fail-merge.sql"), "{body}");
}

#[test]
fn reads_and_unlisted_tables_are_quiet() {
    let body = messages(
        COLUMNS,
        &[
            "schema.sql",
            "pass.ts",
            "pass-invoices.ts",
            "pass-outside.ts",
        ],
    );
    assert!(body.is_empty(), "{body:?}");
}

#[test]
fn empty_list_keeps_todays_behavior() {
    let body = messages("{}", &["schema.sql", "fail-update.ts"]);
    assert!(body.is_empty(), "{body:?}");
}

#[test]
fn case_folds_either_way() {
    let upper_sql = messages(COLUMNS, &["schema.sql", "fail-upper.sql"]);
    assert!(
        upper_sql
            .iter()
            .any(|message| message.to_ascii_lowercase().contains("`orders.updated_at`")),
        "{upper_sql:?}"
    );
    let upper_config = messages(
        "triggerMaintainedColumns: [UPDATED_AT]\n",
        &["schema.sql", "fail-update.ts"],
    );
    assert!(
        upper_config
            .iter()
            .any(|message| message.contains("`orders.updated_at`")),
        "{upper_config:?}"
    );
}

#[test]
fn generated_and_listed_uses_the_generated_message() {
    let body = messages(
        "triggerMaintainedColumns: [updated_at, created_at]\n",
        &["schema.sql", "fail-generated.ts"],
    )
    .join("\n");
    assert!(
        body.contains("do not write generated column `items.created_at`"),
        "{body}"
    );
    assert!(
        !body.contains("trigger-maintained column `items.created_at`"),
        "{body}"
    );
}

#[test]
fn one_statement_that_writes_twice_is_one_finding() {
    let body = messages(COLUMNS, &["schema.sql", "fail-twice.ts"]);
    assert_eq!(body.len(), 1, "{body:?}");
}

#[test]
fn each_listed_column_is_its_own_finding() {
    let body = messages(
        "triggerMaintainedColumns: [updated_at, modified_at]\n",
        &["schema.sql", "fail-both.ts"],
    );
    assert_eq!(body.len(), 2, "{body:?}");
    assert!(body.iter().any(|message| message.contains("updated_at")));
    assert!(body.iter().any(|message| message.contains("modified_at")));
}

#[test]
fn stale_entry_matches_no_column() {
    let body = messages("triggerMaintainedColumns: [touched_at]\n", &["schema.sql"]);
    assert_eq!(body.len(), 1, "{body:?}");
    assert!(
        body[0].contains(
            "stale triggerMaintainedColumns entry: `touched_at` matches no column in schema SQL"
        ),
        "{body:?}"
    );
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
