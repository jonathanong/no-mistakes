use super::normalize;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::tokenizer::Tokenizer;

fn run(sql: &str) -> Vec<sqlparser::tokenizer::TokenWithSpan> {
    let mut tokens = Tokenizer::new(&PostgreSqlDialect {}, sql)
        .tokenize_with_location()
        .unwrap();
    normalize(&mut tokens);
    tokens
}

fn text(sql: &str) -> String {
    run(sql).iter().map(|t| t.token.to_string()).collect()
}

#[test]
fn rewrites_no_key_update_to_update() {
    assert_eq!(
        text("SELECT 1 FROM t FOR NO KEY UPDATE OF a NOWAIT"),
        "SELECT 1 FROM t FOR UPDATE OF a NOWAIT"
    );
    assert_eq!(
        text("SELECT 1 FROM t for  no\n key update"),
        "SELECT 1 FROM t for   update"
    );
}

#[test]
fn rewrites_key_share_to_share_including_at_end_of_input() {
    assert_eq!(text("SELECT 1 FROM t FOR KEY SHARE"), "SELECT 1 FROM t FOR SHARE");
    assert_eq!(
        text("SELECT 1 FROM t FOR KEY SHARE OF a, b"),
        "SELECT 1 FROM t FOR SHARE OF a, b"
    );
}

#[test]
fn keeps_original_spans_of_surviving_tokens() {
    let sql = "SELECT 1 FROM t FOR NO KEY UPDATE";
    let tokens = run(sql);
    let update = tokens.last().unwrap();
    assert_eq!(update.token.to_string(), "UPDATE");
    assert_eq!(update.span.start.column, 28);
}

#[test]
fn leaves_other_text_unchanged() {
    for sql in [
        "SELECT 1 FROM t FOR UPDATE",
        "SELECT 1 FROM t FOR SHARE",
        "SELECT 1 FROM t FOR NO KEY",
        "SELECT 1 FROM t FOR NO",
        "SELECT 1 FROM t FOR KEY",
        "SELECT 1 FROM t FOR KEY OF a",
        "SELECT 1 FROM t FOR \"NO\" KEY UPDATE",
        "SELECT 1 FROM t FOR 1",
        "SELECT no, key, update FROM t",
        "SELECT 1 FROM t",
    ] {
        assert_eq!(
            text(sql),
            Tokenizer::new(&PostgreSqlDialect {}, sql)
                .tokenize_with_location()
                .unwrap()
                .iter()
                .map(|t| t.token.to_string())
                .collect::<String>(),
            "{sql}"
        );
    }
}
