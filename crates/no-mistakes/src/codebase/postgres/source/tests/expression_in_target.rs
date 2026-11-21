use super::super::{parse_postgres_source, PostgresSqlSource, PostgresSqlStatementKind};

#[test]
fn unnest_membership_is_an_unsupported_expression_form() {
    let result = parse_postgres_source(&PostgresSqlSource {
        sql: "SELECT id FROM orders WHERE id IN UNNEST(ARRAY[1])".into(),
        file_name: None,
    });
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let PostgresSqlStatementKind::Select { query } = &result.statements[0].facts else {
        panic!("select expected");
    };
    assert!(
        query
            .unsupported
            .iter()
            .any(|item| item.reason == "expression form"),
        "{:?}",
        query.unsupported
    );
}
