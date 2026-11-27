use crate::bound_expr::{canonical_bound_expr, conjuncts_of};
use crate::parse::{parse_postgres_expression, parse_postgres_sql_lenient_with_sources};

#[test]
fn parse_postgres_expression_handles_aliases_and_rejects_non_scalar_queries() {
    let aliased = parse_postgres_expression("lower(email) AS normalized").expect("aliased expr");
    assert_eq!(canonical_bound_expr(&aliased).to_string(), "lower(email)");
    assert!(parse_postgres_expression("1; DROP TABLE accounts").is_none());
    assert!(parse_postgres_expression("1 UNION SELECT 2").is_none());
    assert!(parse_postgres_expression("*").is_none());
    assert!(parse_postgres_expression("a, b").is_none());
}

#[test]
fn conjuncts_of_flattens_nested_and_predicates() {
    let expr = parse_postgres_expression("((a = 1) AND (b = 2))").expect("nested predicate");
    let mut out = Vec::new();
    conjuncts_of(&expr, &mut out);
    let rendered: Vec<_> = out.iter().map(ToString::to_string).collect();
    assert_eq!(rendered, ["a = 1", "b = 2"]);
}

#[test]
fn malformed_loop_headers_remain_recovery_failures() {
    for sql in [
        "DO $$ BEGIN FOR x IN 1..10; END $$;",
        "DO $$ BEGIN WHILE a > 0; END $$;",
        "DO $$ BEGIN <<123>> LOOP SELECT 1; END LOOP; END $$;",
        "DO $$ BEGIN IF a > THEN LOOP SELECT 1; END LOOP; END $$;",
        "DO $$ BEGIN LOOP IF a > THEN SELECT 1; END IF; END LOOP; END $$;",
    ] {
        let located = parse_postgres_sql_lenient_with_sources(sql, sql);
        assert!(
            located.iter().any(|item| item.recovery_failure.is_some()),
            "expected recovery failure for: {sql}"
        );
    }
}
