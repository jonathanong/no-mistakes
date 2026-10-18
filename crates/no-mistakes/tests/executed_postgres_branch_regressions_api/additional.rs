use super::*;

#[test]
fn wrapped_builder_aliases_preserve_canonical_update_alternatives() {
    let rows = extract("control-flow-wrapper-update-baselines.ts");
    assert_eq!(rows.len(), 3, "{rows:?}");
    assert!(
        rows.iter()
            .filter(|row| row.line == 10)
            .all(|row| row.complete),
        "{rows:?}"
    );
    assert!(
        !rows.iter().find(|row| row.line == 18).unwrap().complete,
        "{rows:?}"
    );
    let mut sql = rows
        .iter()
        .filter(|row| row.line == 10)
        .map(|row| row.sql_text.as_deref().unwrap())
        .collect::<Vec<_>>();
    sql.sort_unstable();
    assert_eq!(sql, ["SELECT 1", "SELECT 1; DELETE FROM records"]);
}

#[test]
fn nested_sql_builder_without_executors_returns_no_statements() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/executed-source/nested-builder-no-executor.mts");
    let sources = [ExecutedPostgresSource {
        file_name: "synthetic.mts".into(),
        source: std::fs::read_to_string(path).unwrap(),
    }];
    let options = EmbeddedSqlOptions::configured(
        "@data-stores/psql",
        &["query".into(), "read".into(), "write".into()],
    );
    assert!(
        no_mistakes::codebase::postgres::extract_executed_postgres_sql(&sources, &options)
            .unwrap()
            .is_empty()
    );
    assert!(
        extract_executed_postgres_sql_with_imports(&sources, &options, &[])
            .unwrap()
            .is_empty()
    );
}

#[test]
fn temporary_callback_operands_survive_nested_helper_branches() {
    let rows = extract("transient-array-callbacks.mts");
    for annotation in [
        "transient/argument",
        "transient/element",
        "transient/private",
        "transient/worklist",
    ] {
        let matching = rows
            .iter()
            .filter(|row| row.annotation.as_deref() == Some(annotation))
            .collect::<Vec<_>>();
        assert!(
            !matching.is_empty(),
            "missing callback {annotation}: {rows:?}"
        );
        // The append is only visible after the callback executes; finding the
        // lexical SELECT alone would not prove the transient callback survived.
        assert!(
            matching.iter().any(|row| row
                .sql_text
                .as_deref()
                .is_some_and(|text| text.ends_with("; DELETE FROM records"))),
            "{annotation}: {rows:?}"
        );
    }
}

#[test]
fn removed_child_callback_survives_an_earlier_worklist_branch() {
    let rows = extract("transient-replaced-child.mts");
    assert!(
        rows.iter().any(
            |row| row.annotation.as_deref() == Some("transient/replaced-child")
                && row
                    .sql_text
                    .as_deref()
                    .is_some_and(|text| text.ends_with("; DELETE FROM records"))
        ),
        "{rows:?}"
    );
}
