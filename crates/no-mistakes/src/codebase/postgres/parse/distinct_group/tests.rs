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
