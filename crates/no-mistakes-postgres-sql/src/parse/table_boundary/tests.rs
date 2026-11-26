use super::normalize;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::tokenizer::Tokenizer;

#[test]
fn table_boundary_controls_leave_unaffected_tokens_unchanged() {
    let root = crate::test_support::rule_fixture_root("postgres-bounded-statements");
    for fixture in [
        "table-arm-boundary-controls.sql",
        "table-arm-boundary-final.sql",
        "table-arm-boundary-eof.sql",
    ] {
        let sql = std::fs::read_to_string(root.join("sql").join(fixture)).unwrap();
        let mut tokens = Tokenizer::new(&PostgreSqlDialect {}, &sql)
            .tokenize_with_location()
            .unwrap();
        let original = tokens.clone();
        normalize(&mut tokens);
        assert_eq!(
            tokens, original,
            "unsupported spellings and a final TABLE arm need no padding"
        );
    }
}
