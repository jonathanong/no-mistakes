use no_mistakes::codebase::postgres::{
    extract_executed_postgres_sql, extract_executed_postgres_sql_with_imports, EmbeddedSqlOptions,
    ExecutedPostgresSource, ExecutedPostgresSql, TrustedSqlTag,
};
use std::path::PathBuf;

fn extract(imports: bool) -> Vec<ExecutedPostgresSql> {
    let name = "boolean-stringification.ts";
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/executed-source")
        .join(name);
    let sources = [ExecutedPostgresSource {
        file_name: name.into(),
        source: std::fs::read_to_string(path).unwrap(),
    }];
    let options = EmbeddedSqlOptions::configured("@example/db", &[]).with_trusted_sql_tags(&[
        TrustedSqlTag {
            module: "@example/db".into(),
            name: "sql".into(),
        },
    ]);
    if imports {
        extract_executed_postgres_sql_with_imports(&sources, &options, &[]).unwrap()
    } else {
        extract_executed_postgres_sql(&sources, &options).unwrap()
    }
}

#[test]
fn boolean_stringification_recovers_finite_whole_statements() {
    let rows = extract(true);
    for (line, suffix) in [
        (2, "true"),
        (3, "false"),
        (4, "true, false"),
        (5, "true, 1, value, false"),
        (6, "true, false"),
        (7, "true, false"),
        (8, "true, false"),
    ] {
        let row = rows.iter().find(|row| row.line == line).unwrap();
        assert!(row.complete, "{row:?}");
        assert!(!row.only_value_placeholders, "{row:?}");
        assert!(
            row.sql_text
                .as_ref()
                .unwrap()
                .ends_with(&format!("SELECT {suffix}")),
            "{row:?}"
        );
    }
    let alternatives = rows.iter().filter(|row| row.line == 9).collect::<Vec<_>>();
    assert_eq!(alternatives.len(), 2, "{rows:?}");
    assert!(
        alternatives.iter().all(|row| row.complete),
        "{alternatives:?}"
    );
}

#[test]
fn boolean_stringification_preserves_bind_and_opaque_boundaries() {
    let rows = extract(true);
    for (line, suffix) in [
        (13, "sql_placeholder_1, true, false"),
        (16, "sql_placeholder_1, sql_placeholder_2"),
    ] {
        let row = rows.iter().find(|row| row.line == line).unwrap();
        assert!(!row.complete && row.only_value_placeholders, "{row:?}");
        assert!(row.sql_text.as_ref().unwrap().ends_with(suffix), "{row:?}");
    }
    for line in [17, 19, 21] {
        let row = rows.iter().find(|row| row.line == line).unwrap();
        assert!(!row.complete && !row.only_value_placeholders, "{row:?}");
    }
    // Precise primitive recovery remains opt-in through import-aware evaluation.
    let legacy = extract(false);
    assert!(legacy.iter().any(|row| row.line == 2), "{legacy:?}");
    assert!(
        legacy
            .iter()
            .filter(|row| row.line <= 9)
            .all(|row| !row.complete),
        "{legacy:?}"
    );
}
