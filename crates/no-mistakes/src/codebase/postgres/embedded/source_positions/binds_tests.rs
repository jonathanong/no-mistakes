use super::super::{extract_embedded_sql_from_source, EmbeddedSqlOptions};

#[test]
fn generated_binds_keep_following_tokens_on_their_physical_source_lines() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/source-position-binds.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = extract_embedded_sql_from_source(
        &path,
        &source,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    );
    assert_eq!(facts.calls.len(), 2);
    for (call, expected_sql) in facts.calls.iter().zip([
        "SELECT $1 AS first, $2 AS second, * FROM entries OFFSET 1",
        "SELECT $1 AS first,\n    $2 AS second, * FROM entries OFFSET 2",
    ]) {
        let sql = call.sql_text.as_deref().unwrap();
        assert_eq!(sql, expected_sql);
        let star = sql.find('*').unwrap();
        let sql_line = sql[..star].bytes().filter(|byte| *byte == b'\n').count() as u32 + 1;
        let sql_column = sql[..star].rsplit('\n').next().unwrap().chars().count() as u32 + 1;
        let position = call
            .sql_source_positions
            .iter()
            .rfind(|position| (position.sql_line, position.sql_column) <= (sql_line, sql_column))
            .unwrap();
        let physical = if sql.ends_with("OFFSET 1") { 9 } else { 16 };
        assert_eq!(
            position.source_line + sql_line - position.sql_line,
            physical
        );
    }
    let statements =
        crate::codebase::postgres::collect::dml::embedded_call_facts(&facts, false, false);
    let lines: Vec<_> = statements
        .iter()
        .flat_map(|statement| &statement.selects)
        .flat_map(|select| &select.star_projections)
        .map(|star| star.line)
        .collect();
    assert_eq!(lines, [9, 16]);
}
