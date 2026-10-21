use super::*;
use crate::codebase::postgres::EmbeddedSqlCall;

fn call(sql: &str) -> EmbeddedSqlCall {
    EmbeddedSqlCall {
        sql_text: Some(sql.to_string()),
        kind: EmbeddedSqlKind::Dynamic,
        ..EmbeddedSqlCall::default()
    }
}

#[test]
fn relation_placeholders_are_identifier_interpolations() {
    assert_eq!(
        identifier_placeholder(&call("SELECT id FROM sql_placeholder_1.users")),
        Some(true)
    );
    assert_eq!(
        identifier_placeholder(&call("SELECT id FROM users WHERE id = sql_placeholder_1")),
        Some(false)
    );
    assert_eq!(identifier_placeholder(&call("not sql")), None);
    assert_eq!(
        identifier_placeholder(&EmbeddedSqlCall {
            kind: EmbeddedSqlKind::Dynamic,
            ..EmbeddedSqlCall::default()
        }),
        None
    );
    assert!(overlapping((1, 4), (3, 6)));
    assert!(!overlapping((1, 2), (2, 4)));
}
