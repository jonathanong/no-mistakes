use super::separate_distinct_grouping;

#[test]
fn saved_distinct_rollup_and_cube_keep_literals_and_parse() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-sql-shape-policy/fixture/review-followups/sql/distinct-grouping.sql",
    );
    let sql = std::fs::read_to_string(path).unwrap();
    let rewritten = separate_distinct_grouping(&sql);
    assert!(rewritten.contains("DISTINCT,ROLLUP (())"));
    assert!(rewritten.contains("DISTINCT,\nROLLUP (())"));
    assert!(rewritten.contains("DISTINCT,CUBE (())"));
    assert!(rewritten.contains("-- GROUP BY DISTINCT ROLLUP (())"));
    assert!(rewritten.contains("/* GROUP BY DISTINCT CUBE (()) */"));
    assert!(rewritten.contains("'GROUP BY DISTINCT ROLLUP (())'"));
    assert!(rewritten.contains("$$GROUP BY DISTINCT CUBE (())$$"));
    assert!(rewritten.contains("$body$GROUP BY DISTINCT ROLLUP (())$body$"));
    assert!(rewritten.contains("SELECT DISTINCT id FROM orders"));
    assert!(crate::codebase::postgres::parse_postgres_sql(&sql).is_ok());
}

#[test]
fn opaque_tails_and_word_prefixes_do_not_insert_a_comma() {
    for sql in [
        "SELECT ' GROUP BY DISTINCT ROLLUP (())",
        "SELECT /* GROUP BY DISTINCT ROLLUP (())",
        "SELECT -- GROUP BY DISTINCT ROLLUP (())",
        "SELECT grouping FROM orders GROUP BY\rid",
        "SELECT 'doubled''quote' FROM orders",
    ] {
        assert_eq!(separate_distinct_grouping(sql), sql, "{sql}");
    }
}

#[test]
fn unclosed_dollar_quote_consumes_the_rest_of_the_statement() {
    let sql = "SELECT $tag$ GROUP BY DISTINCT ROLLUP (())";
    assert_eq!(separate_distinct_grouping(sql), sql);
}

#[test]
fn comments_between_distinct_and_rollup_are_whitespace() {
    for sql in [
        "SELECT 1 GROUP BY DISTINCT /* note */ ROLLUP (())",
        "SELECT 1 GROUP BY DISTINCT -- note\nROLLUP (())",
        "SELECT 1 GROUP BY DISTINCT/* note */CUBE (())",
    ] {
        let rewritten = separate_distinct_grouping(sql);
        assert_ne!(rewritten, sql, "{sql}");
        assert_eq!(rewritten.matches('\n').count(), sql.matches('\n').count());
        assert!(
            crate::codebase::postgres::parse_postgres_sql(sql).is_ok(),
            "{sql}"
        );
    }
}

#[test]
fn unterminated_comment_in_the_gap_is_not_rewritten() {
    let sql = "SELECT 1 GROUP BY DISTINCT /* ROLLUP (())";
    assert_eq!(separate_distinct_grouping(sql), sql);
    let sql = "SELECT 1 GROUP BY DISTINCT -x";
    assert_eq!(separate_distinct_grouping(sql), sql);
}

#[test]
fn lenient_parsing_rewrites_distinct_grouping() {
    let sql = "DO $$ BEGIN NULL; END $$;\nSELECT 1 FROM orders GROUP BY DISTINCT ROLLUP (());";
    assert!(!crate::codebase::postgres::parse::parse_postgres_sql_lenient(sql).is_empty());
    assert_eq!(
        crate::codebase::postgres::parse::parse_postgres_sql_lenient(sql)
            .iter()
            .filter(|s| matches!(s, sqlparser::ast::Statement::Query(_)))
            .count(),
        1
    );
}

#[test]
fn escape_strings_and_dollar_identifiers_are_respected() {
    // `\'` does not close an E string; `foo$tag$` is an identifier.
    let escaped = "SELECT E'\\' GROUP BY DISTINCT ROLLUP (())'";
    assert_eq!(separate_distinct_grouping(escaped), escaped);
    let plain = "SELECT 'x\\' GROUP BY DISTINCT ROLLUP (())";
    assert_ne!(separate_distinct_grouping(plain), plain);
    let identifier = "SELECT foo$tag$ FROM t GROUP BY DISTINCT ROLLUP (())";
    assert_ne!(separate_distinct_grouping(identifier), identifier);
}
