use super::*;
use no_mistakes::codebase::postgres::{
    extract_executed_postgres_sql_with_imports, EmbeddedSqlOptions, ExecutedPostgresSource,
};

fn variant_source(name: &str) -> (PathBuf, String) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-offset/variants")
        .join(name);
    let source = std::fs::read_to_string(root.join("src/query.ts")).unwrap();
    (root, source)
}

#[test]
fn named_executor_helper_offset_is_fully_judged() {
    // The #2199 rewrite names the executor inside the helper. The offset is judged
    // and the executed statement has no incomplete reason.
    let (root, source) = variant_source("named-executor-helper");
    let mut expected = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            line.split_once("// findings: ")
                .map(|(_, target)| (index + 1, target.trim().to_string()))
        })
        .collect::<Vec<_>>();
    expected.sort();
    assert!(!expected.is_empty(), "fixture must expect a judged finding");
    let report = check("postgres-no-offset", "named-executor-helper");
    let mut actual = report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| {
            (
                finding["line"].as_u64().unwrap() as usize,
                finding["target"]
                    .as_str()
                    .unwrap()
                    .split('#')
                    .next()
                    .unwrap()
                    .to_string(),
            )
        })
        .collect::<Vec<_>>();
    actual.sort();
    assert_eq!(actual, expected, "{report}");
    assert!(
        actual.iter().all(|(_, target)| target == "offset"),
        "{report}"
    );
    let rows = extract_executed_postgres_sql_with_imports(
        &[ExecutedPostgresSource {
            file_name: root.join("src/query.ts").to_string_lossy().into_owned(),
            source,
        }],
        &EmbeddedSqlOptions::configured("@example/db", &[]),
        &[],
    )
    .unwrap();
    assert!(
        rows.iter().any(|row| {
            row.sql_text
                .as_deref()
                .is_some_and(|sql| sql.contains("OFFSET"))
        }),
        "{rows:?}"
    );
    assert!(
        rows.iter().all(|row| row.incomplete_reason.is_none()),
        "{rows:?}"
    );
}

#[test]
fn passed_executor_value_is_not_a_judged_offset() {
    // `run(read, account)` hides the statement. The report is empty under
    // unanalyzableSql: fail, so there is no judged offset to autofix.
    let report = check("postgres-no-offset", "passed-executor-value");
    let findings = report["rules"].as_array().unwrap();
    assert!(findings.is_empty(), "{report}");
}
