use super::*;
use crate::codebase::postgres::{SqlColumnClause, SqlStatementFileFacts};

fn columns(facts: &SqlStatementFileFacts) -> Vec<(String, SqlColumnClause)> {
    facts
        .selects
        .iter()
        .flat_map(|select| &select.column_uses)
        .chain(&facts.mutation_column_uses)
        .map(|fact| (fact.column.clone(), fact.clause))
        .collect()
}

#[test]
fn recovered_binds_are_omitted_but_literal_marker_columns_survive() {
    let root = root();
    let ts = root.join("bind-column-uses.ts");
    let sql = root.join("sql/bind-column-uses.sql");
    let profile = EmbeddedSqlOptions {
        import_specifier: "@db".into(),
        ..Default::default()
    };
    let sources = crate::codebase::rules::source_store_for_files(&[ts.clone(), sql.clone()]);
    let facts =
        crate::codebase::check_facts::collect_check_facts_with_graph_files_playwright_and_sources(
            &root,
            vec![ts.clone(), sql.clone()],
            vec![],
            CheckFactPlan {
                postgres_dml: true,
                postgres_sql_include: vec!["**/*.sql".into()],
                embedded_sql: true,
                embedded_sql_options: vec![profile.clone()],
                ..Default::default()
            },
            None,
            sources,
        );
    let embedded = facts.postgres_statements(&ts, Some(&profile)).unwrap();
    assert_eq!(embedded.len(), 9);
    use SqlColumnClause::{Join, OrderBy, Where};
    let expected = [
        vec![
            ("tenant_id", Where),
            ("sql_placeholder_1", Where),
            ("sql_placeholder_1", Where),
        ],
        vec![("tenant_id", Where), ("sql_placeholder_1", Where)],
        vec![("tenant_id", Where), ("sql_placeholder_1", Where)],
        vec![("sql_placeholder_1", Where), ("sql_placeholder_1", OrderBy)],
        vec![
            ("tenant_id", Where),
            ("id", Join),
            ("id", Join),
            ("sql_placeholder_1", Join),
        ],
        vec![("sql_placeholder_1", Where)],
        vec![],
        vec![("id", OrderBy)],
        vec![("id", OrderBy)],
    ];
    for (fact, expected) in embedded.iter().zip(expected) {
        assert!(!fact.parse_failed);
        assert_eq!(
            columns(fact),
            expected
                .into_iter()
                .map(|(column, clause)| (column.to_string(), clause))
                .collect::<Vec<_>>(),
            "{fact:?}"
        );
    }
    let standalone = &facts.postgres_statements(&sql, None).unwrap()[0];
    assert_eq!(
        columns(standalone),
        vec![
            ("sql_placeholder_1".into(), Where),
            ("sql_placeholder_1".into(), Where),
            ("sql_placeholder_1".into(), OrderBy),
            ("sql_placeholder_1".into(), Where),
            ("sql_placeholder_1".into(), Where)
        ]
    );
}
