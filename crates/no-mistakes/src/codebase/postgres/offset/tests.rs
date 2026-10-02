fn statement_offsets(statement: &sqlparser::ast::Statement, out: &mut Vec<OffsetUse>) {
    let mut collector = super::OffsetCollector::default();
    let _ = sqlparser::ast::Visit::visit(statement, &mut collector);
    collector.uses.sort_by_key(|fact| (fact.line, fact.column));
    out.extend(collector.uses.into_iter().map(|fact| fact.kind));
}

use super::{sql_has_offset_clause, sql_offset_uses, OffsetUse};

#[test]
fn offset_keyword_is_detected() {
    assert!(sql_has_offset_clause("SELECT id FROM posts OFFSET 10").unwrap());
    assert!(
        sql_has_offset_clause("SELECT id FROM posts LIMIT 10 OFFSET sql_placeholder_1").unwrap()
    );
}

#[test]
fn limit_without_offset_is_clean() {
    assert!(!sql_has_offset_clause("SELECT id FROM posts ORDER BY id DESC LIMIT 11").unwrap());
}

#[test]
fn offset_in_string_literal_is_ignored() {
    assert!(!sql_has_offset_clause(
        "INSERT INTO examples (body) VALUES ('offset by a travel credit')"
    )
    .unwrap());
}

#[test]
fn subquery_and_cte_offsets_are_detected() {
    assert!(
        sql_has_offset_clause("SELECT * FROM (SELECT id FROM posts OFFSET 1) AS page").unwrap()
    );
    assert!(sql_has_offset_clause(
        "WITH page AS (SELECT id FROM posts OFFSET 1) SELECT * FROM page"
    )
    .unwrap());
}

#[test]
fn insert_select_offset_is_detected() {
    assert!(sql_has_offset_clause("INSERT INTO t SELECT * FROM u OFFSET 5").unwrap());
}

#[test]
fn unparseable_sql_returns_error() {
    let error = sql_has_offset_clause("SELECT id FROM posts OFFSET").expect_err("unparseable");
    assert!(!error.message.is_empty());
}

#[test]
fn non_query_statements_are_clean() {
    assert!(!sql_has_offset_clause("CREATE TABLE t (id int)").unwrap());
    assert!(!sql_has_offset_clause("DROP TABLE t").unwrap());
}

#[test]
fn union_offset_is_detected() {
    assert!(sql_has_offset_clause("SELECT id FROM a UNION SELECT id FROM b OFFSET 2").unwrap());
}

#[test]
fn in_subquery_offset_is_detected() {
    assert!(sql_has_offset_clause(
        "SELECT id FROM posts WHERE id IN (SELECT id FROM other OFFSET 1)"
    )
    .unwrap());
}

#[test]
fn join_derived_offset_is_detected() {
    assert!(sql_has_offset_clause(
        "SELECT * FROM t JOIN (SELECT id FROM u OFFSET 1) AS page ON true"
    )
    .unwrap());
}

#[test]
fn parenthesized_predicate_without_offset_is_clean() {
    assert!(!sql_has_offset_clause("SELECT * FROM t WHERE (id = ANY($1))").unwrap());
}

#[test]
fn union_arm_offset_is_detected() {
    assert!(sql_has_offset_clause(
        "SELECT id FROM a WHERE id IN (SELECT x FROM t OFFSET 1) UNION SELECT id FROM b"
    )
    .unwrap());
}

#[test]
fn parenthesized_union_offset_is_detected() {
    assert!(
        sql_has_offset_clause("(SELECT id FROM posts OFFSET 1) UNION SELECT id FROM other")
            .unwrap()
    );
}

#[test]
fn scalar_subquery_offset_is_detected() {
    assert!(sql_has_offset_clause(
        "SELECT id FROM posts WHERE id = (SELECT id FROM other OFFSET 1)"
    )
    .unwrap());
}

#[test]
fn binary_and_unary_predicate_offsets_are_detected() {
    assert!(sql_has_offset_clause(
        "SELECT id FROM posts WHERE live AND id IN (SELECT id FROM other OFFSET 1)"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "SELECT id FROM posts WHERE NOT (id IN (SELECT id FROM other OFFSET 1))"
    )
    .unwrap());
}

#[test]
fn exists_subquery_offset_is_detected() {
    assert!(
        sql_has_offset_clause("SELECT * FROM t WHERE EXISTS (SELECT 1 FROM u OFFSET 1)").unwrap()
    );
}

#[test]
fn projection_subquery_offset_is_detected() {
    assert!(sql_has_offset_clause("SELECT (SELECT 1 FROM u OFFSET 1) FROM t").unwrap());
}

#[test]
fn join_on_exists_offset_is_detected() {
    assert!(
        sql_has_offset_clause("SELECT * FROM t JOIN u ON EXISTS (SELECT 1 FROM v OFFSET 1)")
            .unwrap()
    );
    assert!(sql_has_offset_clause(
        "SELECT * FROM t LEFT JOIN u ON EXISTS (SELECT 1 FROM v OFFSET 1)"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "SELECT * FROM t RIGHT JOIN u ON EXISTS (SELECT 1 FROM v OFFSET 1)"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "SELECT * FROM t FULL JOIN u ON EXISTS (SELECT 1 FROM v OFFSET 1)"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "SELECT * FROM t INNER JOIN u ON EXISTS (SELECT 1 FROM v OFFSET 1)"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "SELECT * FROM t LEFT OUTER JOIN u ON EXISTS (SELECT 1 FROM v OFFSET 1)"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "SELECT * FROM t RIGHT OUTER JOIN u ON EXISTS (SELECT 1 FROM v OFFSET 1)"
    )
    .unwrap());
}

#[test]
fn order_by_subquery_offset_is_detected() {
    assert!(
        sql_has_offset_clause("SELECT id FROM t ORDER BY (SELECT id FROM u OFFSET 1 LIMIT 1)")
            .unwrap()
    );
    assert!(!sql_has_offset_clause("SELECT id FROM t ORDER BY id").unwrap());
    assert!(!sql_has_offset_clause("SELECT id FROM t ORDER BY ALL").unwrap());
}

#[test]
fn create_table_and_view_offsets_are_detected() {
    assert!(sql_has_offset_clause("CREATE TABLE page AS SELECT * FROM posts OFFSET 10").unwrap());
    assert!(sql_has_offset_clause("CREATE VIEW page AS SELECT * FROM posts OFFSET 10").unwrap());
}

#[test]
fn having_exists_offset_is_detected() {
    assert!(sql_has_offset_clause(
        "SELECT id FROM t GROUP BY id HAVING EXISTS (SELECT 1 FROM u OFFSET 1)"
    )
    .unwrap());
}

#[test]
fn join_without_on_subquery_is_clean() {
    assert!(!sql_has_offset_clause("SELECT * FROM t JOIN u USING (id)").unwrap());
    assert!(!sql_has_offset_clause("SELECT * FROM t CROSS JOIN u").unwrap());
}

#[test]
fn update_and_delete_subquery_offsets_are_detected() {
    assert!(
        sql_has_offset_clause("UPDATE users SET rank = (SELECT rank FROM rankings OFFSET 1)")
            .unwrap()
    );
    assert!(
        sql_has_offset_clause("DELETE FROM users WHERE id IN (SELECT id FROM stale OFFSET 1)")
            .unwrap()
    );
    assert!(sql_has_offset_clause(
        "UPDATE users SET rank = 1 WHERE id IN (SELECT id FROM stale OFFSET 1)"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "UPDATE users SET rank = 1 FROM (SELECT id FROM stale OFFSET 1) AS s WHERE users.id = s.id"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "DELETE FROM users USING (SELECT id FROM stale OFFSET 1) AS s WHERE users.id = s.id"
    )
    .unwrap());
}

#[test]
fn values_row_subquery_offset_is_detected() {
    assert!(sql_has_offset_clause(
        "INSERT INTO t(id) VALUES ((SELECT id FROM u OFFSET 1 LIMIT 1))"
    )
    .unwrap());
    assert!(sql_has_offset_clause("VALUES ((SELECT id FROM u OFFSET 1 LIMIT 1))").unwrap());
}

#[test]
fn explain_and_copy_offset_is_detected() {
    assert!(sql_has_offset_clause("EXPLAIN ANALYZE SELECT id FROM posts OFFSET 10").unwrap());
    assert!(!sql_has_offset_clause("EXPLAIN SELECT id FROM posts OFFSET 10").unwrap());
    assert!(!sql_has_offset_clause("EXPLAIN SELECT id FROM posts LIMIT 10").unwrap());
    assert!(sql_has_offset_clause("COPY (SELECT id FROM posts OFFSET 1) TO STDOUT").unwrap());
}

#[test]
fn returning_and_on_conflict_offsets_are_detected() {
    assert!(sql_has_offset_clause(
        "INSERT INTO audit DEFAULT VALUES RETURNING (SELECT id FROM pages OFFSET 1 LIMIT 1)"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "UPDATE users SET rank = 1 RETURNING (SELECT id FROM pages OFFSET 1 LIMIT 1)"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "DELETE FROM users RETURNING (SELECT id FROM pages OFFSET 1 LIMIT 1)"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "INSERT INTO t(id, value) VALUES (1, 1) ON CONFLICT (id) DO UPDATE SET value = (SELECT value FROM u OFFSET 1 LIMIT 1)"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "INSERT INTO t(id) VALUES (1) ON CONFLICT (id) DO UPDATE SET id = 1 WHERE id IN (SELECT id FROM u OFFSET 1)"
    )
    .unwrap());
}

#[test]
fn nested_join_group_by_distinct_and_limit_offsets_are_detected() {
    assert!(sql_has_offset_clause(
        "SELECT * FROM (a JOIN (SELECT * FROM b OFFSET 1) AS page ON true) AS joined"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "SELECT count(*) FROM t GROUP BY (SELECT id FROM pages OFFSET 1 LIMIT 1)"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "SELECT DISTINCT ON ((SELECT id FROM pages OFFSET 1 LIMIT 1)) id FROM t"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "SELECT * FROM t LIMIT (SELECT id FROM limits OFFSET 1 LIMIT 1)"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "SELECT count(*) OVER w FROM t WINDOW w AS (ORDER BY (SELECT id FROM u OFFSET 1 LIMIT 1))"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "SELECT count(*) OVER w FROM t WINDOW w AS (PARTITION BY (SELECT id FROM u OFFSET 1 LIMIT 1))"
    )
    .unwrap());
}

#[test]
fn function_args_and_modifying_cte_offsets_are_detected() {
    assert!(
        sql_has_offset_clause("SELECT COALESCE((SELECT id FROM pages OFFSET 1 LIMIT 1), 0)")
            .unwrap()
    );
    assert!(!sql_has_offset_clause("SELECT COUNT(*) FROM t").unwrap());
    assert!(sql_has_offset_clause(
        "WITH changed AS (UPDATE users SET rank = (SELECT rank FROM rankings OFFSET 1) RETURNING id) SELECT * FROM changed"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "WITH removed AS (DELETE FROM users WHERE id IN (SELECT id FROM stale OFFSET 1) RETURNING id) SELECT * FROM removed"
    )
    .unwrap());
    assert!(sql_has_offset_clause(
        "WITH added AS (INSERT INTO t SELECT id FROM u OFFSET 1 RETURNING id) SELECT * FROM added"
    )
    .unwrap());
    assert!(!sql_has_offset_clause("SELECT CURRENT_TIMESTAMP").unwrap());
    assert!(sql_has_offset_clause(
        "SELECT * FROM generate_series(1, (SELECT id FROM limits OFFSET 1 LIMIT 1)) AS g"
    )
    .unwrap());
}

#[test]
fn sql_file_splitting_covers_comments_quotes_and_other_statements() {
    let uses = super::sql_file_offset_uses(
        "/* header\n still */\nCREATE TABLE t (id int);\nSELECT 'a\nb';\nSELECT id FROM posts OFFSET 1;",
    );
    assert_eq!(uses, vec![(6, OffsetUse::Other)]);
    assert!(super::sql_file_offset_uses("/* unterminated\nSELECT 1").is_empty());
    assert!(super::sql_file_offset_uses("SELECT 'unterminated\n").is_empty());
    let dollar = super::sql_file_offset_uses(
        "SELECT $tag$one\ntwo$tag$;\nSELECT $1;\nSELECT id FROM posts OFFSET 2;",
    );
    assert_eq!(dollar, vec![(4, OffsetUse::Other)]);
    assert!(super::sql_file_offset_uses("SELECT $$unterminated\n").is_empty());
}

#[test]
fn parenthesized_zero_offset_is_still_zero() {
    assert_eq!(
        sql_offset_uses("SELECT id FROM posts OFFSET (0)").unwrap(),
        vec![OffsetUse::Zero]
    );
    assert_eq!(
        sql_offset_uses("SELECT id FROM posts OFFSET ((0))").unwrap(),
        vec![OffsetUse::Zero]
    );
}

#[test]
fn group_by_all_window_reference_and_table_are_clean() {
    assert!(!sql_has_offset_clause("SELECT id FROM posts GROUP BY ALL").unwrap());
    assert!(!sql_has_offset_clause("SELECT id FROM posts UNION TABLE other").unwrap());
    let sql = "SELECT count(*) OVER w FROM posts WINDOW w AS prev";
    let statements =
        sqlparser::parser::Parser::parse_sql(&sqlparser::dialect::GenericDialect {}, sql)
            .unwrap_or_else(|error| panic!("{sql}: {error}"));
    let mut uses = Vec::new();
    for statement in &statements {
        statement_offsets(statement, &mut uses);
    }
    assert!(uses.is_empty(), "{uses:?}");
}

#[test]
fn dialect_limit_forms_are_collected() {
    for sql in [
        "SELECT id FROM posts ORDER BY ALL",
        "SELECT id FROM posts LIMIT 10, 2",
        "SELECT id FROM posts LIMIT 10 BY id OFFSET 1",
        "SELECT id FROM posts LIMIT 10 BY (SELECT id FROM other OFFSET 1) OFFSET 0",
    ] {
        let statements =
            sqlparser::parser::Parser::parse_sql(&sqlparser::dialect::GenericDialect {}, sql)
                .unwrap_or_else(|error| panic!("{sql}: {error}"));
        let mut uses = Vec::new();
        for statement in &statements {
            statement_offsets(statement, &mut uses);
        }
        assert!(
            !uses.is_empty() || !sql.contains("OFFSET"),
            "{sql}: {uses:?}"
        );
    }
}

#[test]
fn zero_and_other_offsets_are_collected_separately() {
    assert_eq!(
        sql_offset_uses("SELECT id FROM posts OFFSET 0").unwrap(),
        vec![OffsetUse::Zero]
    );
    assert_eq!(
        sql_offset_uses("SELECT id FROM posts OFFSET 0 ROWS").unwrap(),
        vec![OffsetUse::Zero]
    );
    assert_eq!(
        sql_offset_uses("SELECT id FROM posts OFFSET $1").unwrap(),
        vec![OffsetUse::Other]
    );
    let uses =
        sql_offset_uses("SELECT * FROM (SELECT id FROM posts OFFSET 0) o OFFSET 40").unwrap();
    assert_eq!(uses.len(), 2, "{uses:?}");
    assert!(uses.contains(&OffsetUse::Zero));
    assert!(uses.contains(&OffsetUse::Other));
}

#[test]
fn spanless_recovery_uses_offset_keyword_and_empty_source_is_safe() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-offset/fixture/review-followups/db/spanless.sql");
    let sql = std::fs::read_to_string(path).unwrap();
    let tokens = super::super::parse::unicode::tokenize_raw_unicode(&sql)
        .into_iter()
        .map(|token| token.token)
        .collect();
    let statements = sqlparser::parser::Parser::new(&sqlparser::dialect::PostgreSqlDialect {})
        .with_tokens(tokens)
        .parse_statements()
        .unwrap();
    let facts = super::offset_facts(&sql, &statements);
    assert_eq!(facts.len(), 1);
    assert_eq!((facts[0].line, facts[0].kind), (2, OffsetUse::Zero));
    let facts = super::offset_facts("", &statements);
    assert_eq!(facts[0].line, 1);
}

#[test]
fn comment_separated_offset_keeps_the_keyword_column() {
    for sql in [
        "SELECT id\nOFFSET /* note */ 1",
        "SELECT id OFFSET -- note\n1",
        "SELECT id OFFSET /* outer /* inner */ x */ 1",
    ] {
        let statements = super::super::parse::parse_postgres_sql(sql).unwrap();
        let facts = super::offset_facts(sql, &statements);
        let keyword = sql
            .lines()
            .enumerate()
            .find_map(|(index, line)| line.find("OFFSET").map(|column| (index + 1, column + 1)))
            .unwrap();
        assert_eq!(facts.len(), 1, "{sql}");
        assert_eq!((facts[0].line, facts[0].column), keyword, "{sql}");
        assert_eq!(facts[0].kind, OffsetUse::Other);
    }
}

#[test]
fn unclosed_separator_does_not_attach_an_earlier_keyword() {
    let keyword = "SELECT id OFFSET".find("OFFSET").unwrap() + 1;
    for sql in ["SELECT id OFFSET /* note", "SELECT id OFFSET -- note"] {
        assert!(super::locate::resolve(sql, &[(1, keyword)], 0, 1, sql.len()).is_none());
    }
    assert!(super::locate::resolve("SELECT 1", &[], 0, 1, 0).is_none());
}

#[test]
fn recovered_statements_keep_the_outer_offset_keyword() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-no-offset/fixture/review-followups/db/recovered-outer.sql",
    );
    let sql = std::fs::read_to_string(path).unwrap();
    let facts = super::super::statements::extract_sql_statement_facts(&sql);
    assert_eq!(
        facts
            .offset_uses
            .iter()
            .map(|fact| (fact.line, fact.kind))
            .collect::<Vec<_>>(),
        vec![
            (2, OffsetUse::Other),
            (4, OffsetUse::Other),
            (6, OffsetUse::Other),
            (7, OffsetUse::Other),
        ]
    );
    for fact in facts.offset_uses.iter().take(3) {
        let line = sql.lines().nth(fact.line - 1).unwrap();
        assert_eq!(fact.column, line.find("OFFSET").unwrap() + 1);
    }
    assert!(facts.offset_uses[3].column >= 1);
}
