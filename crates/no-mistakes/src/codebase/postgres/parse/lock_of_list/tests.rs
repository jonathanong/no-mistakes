use super::normalize;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::tokenizer::Tokenizer;

fn tokens(sql: &str) -> Vec<sqlparser::tokenizer::TokenWithSpan> {
    Tokenizer::new(&PostgreSqlDialect {}, sql)
        .tokenize_with_location()
        .unwrap()
}

fn text(tokens: &[sqlparser::tokenizer::TokenWithSpan]) -> String {
    tokens.iter().map(|t| t.token.to_string()).collect()
}

#[test]
fn splits_a_list_into_one_clause_per_relation() {
    let mut tokens = tokens("SELECT 1 FROM t FOR NO KEY UPDATE OF a, s.b NOWAIT");
    normalize(&mut tokens);
    assert_eq!(
        text(&tokens),
        "SELECT 1 FROM t FOR NO KEY UPDATE OF a NOWAIT FOR NO KEY UPDATE OF s.b NOWAIT"
    );
}

#[test]
fn leaves_other_for_clauses_unchanged() {
    for sql in [
        "SELECT 1 FROM t FOR UPDATE OF a",
        "SELECT 1 FROM t FOR UPDATE",
        "SELECT 1 FROM t FOR SHARE NOWAIT",
        "SELECT 1 FROM t FOR UPDATE OF a, 1",
        "SELECT 1 FROM t FOR UPDATE OF a,",
        "SELECT 1 FROM t FOR UPDATE OF a, b.",
        "SELECT 1 FROM t FOR OF a, b",
        "SELECT 1 FROM t FOR UPDATE a, b",
        "SELECT 1 FROM t FOR KEY OF a, b",
        "SELECT 1 FROM t FOR",
        "SELECT 1 FROM t",
    ] {
        let mut tokens = tokens(sql);
        let original = tokens.clone();
        normalize(&mut tokens);
        assert_eq!(tokens, original, "{sql}");
    }
}
