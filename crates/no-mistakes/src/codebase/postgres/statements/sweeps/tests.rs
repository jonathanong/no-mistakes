use crate::codebase::postgres::statements::{
    extract_sql_statement_facts, SqlCursorBound, SqlLimitFact, SqlLimitValue, SqlSweepFact,
};

fn limits(sql: &str) -> Vec<(usize, usize, SqlLimitValue)> {
    extract_sql_statement_facts(sql)
        .limit_uses
        .iter()
        .map(|limit| (limit.line, limit.column, limit.value))
        .collect()
}

fn values(sql: &str) -> Vec<SqlLimitValue> {
    extract_sql_statement_facts(sql)
        .limit_uses
        .iter()
        .map(|limit: &SqlLimitFact| limit.value)
        .collect()
}

fn sweeps(sql: &str) -> Vec<SqlSweepFact> {
    extract_sql_statement_facts(sql).sweeps
}

/// `table order-columns | conjunct(cursor columns) ...` for each sweep.
fn shape(sql: &str) -> Vec<String> {
    sweeps(sql)
        .iter()
        .map(|sweep| {
            let conjuncts: Vec<String> = sweep
                .conjuncts
                .iter()
                .map(|conjunct| format!("{}({})", conjunct.text, conjunct.cursor_columns.join(",")))
                .collect();
            format!(
                "{} {} | {}",
                sweep.table,
                sweep.order_columns.join(","),
                conjuncts.join(" ; ")
            )
        })
        .collect()
}

#[test]
fn every_limit_and_fetch_records_its_count_and_position() {
    use SqlLimitValue::{Literal, Other};
    assert_eq!(values("SELECT 1 FROM t LIMIT 500"), [Literal(500)]);
    assert_eq!(values("SELECT 1 FROM t LIMIT $1"), [Other]);
    assert_eq!(values("SELECT 1 FROM t LIMIT (10)"), [Literal(10)]);
    assert_eq!(values("SELECT 1 FROM t LIMIT 1 + 1"), [Other]);
    assert_eq!(values("SELECT 1 FROM t LIMIT 1.5"), [Other]);
    assert_eq!(
        values("SELECT 1 FROM t FETCH FIRST 7 ROWS ONLY"),
        [Literal(7)]
    );
    assert_eq!(values("SELECT 1 FROM t FETCH FIRST ROW ONLY"), [Literal(1)]);
    assert_eq!(
        values("SELECT 1 FROM t FETCH FIRST 5 PERCENT ROWS ONLY"),
        [Other]
    );
    // An unbounded LIMIT is not a cap.
    assert!(values("SELECT 1 FROM t LIMIT NULL").is_empty());
    assert!(values("SELECT 1 FROM t LIMIT ALL").is_empty());
    assert!(values("SELECT 1 FROM t").is_empty());
    // Nested queries, CTEs and mutation subqueries are all found, in source order.
    assert_eq!(
        values(
            "WITH c AS (SELECT id FROM a ORDER BY id LIMIT 1000 FOR UPDATE SKIP LOCKED)\n\
             DELETE FROM a USING c WHERE a.id = c.id AND a.k IN (SELECT k FROM b LIMIT 3)"
        ),
        [Literal(1000), Literal(3)]
    );
    assert_eq!(
        limits("SELECT 1\nFROM t\nLIMIT 5"),
        [(3, 7, Literal(5))],
        "line and column of the count"
    );
}

#[test]
fn a_limited_single_table_query_ordered_by_plain_columns_is_a_sweep() {
    assert_eq!(
        shape("SELECT id FROM orders WHERE id > $1 ORDER BY id LIMIT $2"),
        ["orders id | id > $1(id)"]
    );
    assert_eq!(
        shape("SELECT id FROM invoices ORDER BY id LIMIT 1000"),
        ["invoices id | "]
    );
    assert_eq!(
        shape("SELECT id FROM public.orders o WHERE o.id >= $1::uuid ORDER BY o.id DESC LIMIT 5"),
        ["public.orders id | o.id >= $1::uuid(id)"]
    );
    // Row comparison and the optional-cursor form are cursors; anything else is not.
    assert_eq!(
        shape("SELECT 1 FROM t WHERE (a, b) > ($1, $2) ORDER BY a, b LIMIT 3"),
        ["t a,b | (a, b) > ($1, $2)(a,b)"]
    );
    assert_eq!(
        shape(
            "SELECT 1 FROM t WHERE ($1::uuid IS NULL OR id > $1) AND deleted_at IS NULL \
             ORDER BY id LIMIT $2"
        ),
        ["t id | $1::uuid is null or id > $1(id) ; deleted_at is null()"]
    );
    assert_eq!(
        shape("SELECT 1 FROM t WHERE id > $1 OR $1 IS NULL ORDER BY id LIMIT 3"),
        ["t id | id > $1 or $1 is null(id)"]
    );
    assert_eq!(
        shape("SELECT 1 FROM t WHERE ready_at <= now() AND id = $1 AND created > 5 ORDER BY id LIMIT 3"),
        ["t id | ready_at <= now()() ; id = $1() ; created > 5()"]
    );
    // A cursor value must be a bind parameter, and the column a plain column.
    assert_eq!(
        shape("SELECT 1 FROM t WHERE id > $1 OR k = 1 ORDER BY id LIMIT 3"),
        ["t id | id > $1 or k = 1()"]
    );
    assert_eq!(
        shape("SELECT 1 FROM t WHERE (id, k) > ($1, 5) AND (id, lower(k)) > ($2, $3) ORDER BY id LIMIT 3"),
        ["t id | (id, k) > ($1, 5)() ; (id, lower(k)) > ($2, $3)()"]
    );
    assert_eq!(
        shape("SELECT 1 FROM t WHERE id > $1 + 1 AND lower(k) > $2 AND other.x > $3 ORDER BY id LIMIT 3"),
        ["t id | id > $1 + 1() ; lower(k) > $2() ; other.x > $3()"]
    );
}

#[test]
fn other_queries_are_not_sweeps() {
    for sql in [
        "SELECT 1 FROM t WHERE id > $1 ORDER BY id",
        "SELECT 1 FROM t WHERE id > $1 LIMIT 3",
        "SELECT 1 FROM t WHERE id > $1 ORDER BY lower(id) LIMIT 3",
        "SELECT 1 FROM t WHERE id > $1 ORDER BY 1 LIMIT 3",
        "SELECT 1 FROM t JOIN u ON u.t_id = t.id ORDER BY t.id LIMIT 3",
        "SELECT 1 FROM t, u ORDER BY t.id LIMIT 3",
        "SELECT k, count(*) FROM t GROUP BY k ORDER BY k LIMIT 3",
        "SELECT 1 FROM (SELECT 1 AS id) d ORDER BY id LIMIT 3",
        "SELECT 1 FROM unnest($1::int[]) AS x ORDER BY 1 LIMIT 3",
        "SELECT id FROM a UNION SELECT id FROM b ORDER BY id LIMIT 3",
        "WITH c AS (SELECT 1 AS id) SELECT id FROM c ORDER BY id LIMIT 3",
        "SELECT 1 FROM t ORDER BY other.id LIMIT 3",
    ] {
        assert!(sweeps(sql).is_empty(), "{sql}");
    }
    // A schema-qualified table is a table even when a CTE shares its bare name.
    assert_eq!(
        sweeps("WITH t AS (SELECT 1) SELECT id FROM public.t ORDER BY id LIMIT 3").len(),
        1
    );
}

#[test]
fn sweeps_inside_subqueries_and_ctes_are_found_at_their_relation_line() {
    let found = sweeps(
        "WITH c AS (\n  SELECT id FROM invoices ORDER BY id LIMIT 1000 FOR UPDATE SKIP LOCKED\n)\n\
         DELETE FROM invoices USING c WHERE invoices.id = c.id",
    );
    assert_eq!(found.len(), 1);
    assert_eq!((found[0].table.as_str(), found[0].line), ("invoices", 2));
    assert!(found[0].conjuncts.is_empty());
}

#[test]
fn parenthesized_cursors_are_unwrapped_at_every_depth() {
    for sql in [
        "SELECT 1 FROM t WHERE ((id > $1)) ORDER BY id LIMIT $2",
        "SELECT 1 FROM t WHERE (((id) > ($1))) ORDER BY id LIMIT $2",
        "SELECT 1 FROM t WHERE ((($1 IS NULL) OR ((id > $1)))) ORDER BY id LIMIT $2",
    ] {
        let found = sweeps(sql);
        assert_eq!(found[0].conjuncts.len(), 1, "{sql}");
        assert_eq!(found[0].conjuncts[0].cursor_columns, ["id"], "{sql}");
    }
    let rows = sweeps("SELECT 1 FROM t WHERE (((a, b)) > (($1, $2))) ORDER BY a, b LIMIT 3");
    assert_eq!(rows[0].conjuncts[0].cursor_columns, ["a", "b"]);
}

#[test]
fn recovered_interpolations_are_binds() {
    // `${after}` and `${size}` reach the facts as sql_placeholder_N identifiers.
    assert_eq!(
        shape("SELECT id FROM orders WHERE id > sql_placeholder_1 ORDER BY id LIMIT sql_placeholder_2"),
        ["orders id | id > sql_placeholder_1(id)"]
    );
    assert_eq!(
        shape("SELECT 1 FROM t WHERE (sql_placeholder_1::uuid IS NULL OR id > sql_placeholder_1) ORDER BY id LIMIT 3"),
        ["t id | sql_placeholder_1::uuid is null or id > sql_placeholder_1(id)"]
    );
    assert_eq!(
        shape("SELECT 1 FROM t WHERE (a, b) > (sql_placeholder_1, sql_placeholder_2) ORDER BY a, b LIMIT 3"),
        ["t a,b | (a, b) > (sql_placeholder_1, sql_placeholder_2)(a,b)"]
    );
    // A bind is never the column side of a cursor.
    assert_eq!(
        shape("SELECT 1 FROM t WHERE sql_placeholder_1 > sql_placeholder_2 ORDER BY id LIMIT 3"),
        ["t id | sql_placeholder_1 > sql_placeholder_2()"]
    );
}

#[test]
fn a_cte_body_sees_only_the_ctes_declared_before_it() {
    // The inner `page` is the physical table: a non-recursive CTE cannot see itself.
    let found =
        sweeps("WITH page AS (SELECT id FROM page ORDER BY id LIMIT 3) SELECT id FROM page");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].table, "page");
    // A recursive WITH shows a body itself and its siblings.
    assert!(
        sweeps("WITH RECURSIVE page AS (SELECT id FROM page ORDER BY id LIMIT 3) SELECT 1")
            .is_empty()
    );
    assert!(sweeps(
        "WITH RECURSIVE b AS (SELECT id FROM a ORDER BY id LIMIT 3), a AS (SELECT 1 AS id) SELECT 1 FROM b"
    )
    .is_empty());
    // An earlier CTE is visible to a later body; a later one is not visible to an earlier body.
    assert!(sweeps(
        "WITH a AS (SELECT 1 AS id), b AS (SELECT id FROM a ORDER BY id LIMIT 3) SELECT 1 FROM b"
    )
    .is_empty());
    assert_eq!(
        sweeps(
            "WITH b AS (SELECT id FROM a ORDER BY id LIMIT 3), a AS (SELECT 1 AS id) SELECT 1 FROM b"
        )
        .len(),
        1
    );
    // The main query and its subqueries see every CTE.
    assert!(sweeps(
        "WITH a AS (SELECT 1 AS id) SELECT 1 FROM x WHERE x.id IN (SELECT id FROM a ORDER BY id LIMIT 3)"
    )
    .is_empty());
}

#[test]
fn an_implicit_fetch_is_located_at_its_keyword() {
    use SqlLimitValue::Literal;
    assert_eq!(
        limits("SELECT id FROM t\nORDER BY id\nFETCH FIRST ROW ONLY"),
        [(3, 1, Literal(1))]
    );
    // A comment or string that spells FETCH, and a subquery's own FETCH, are not this query's.
    assert_eq!(
        limits("SELECT 'fetch' FROM t -- fetch\n  FETCH FIRST ROW ONLY"),
        [(2, 3, Literal(1))]
    );
    assert_eq!(
        limits("SELECT * FROM (SELECT 1 FROM t FETCH FIRST ROW ONLY) x\nFETCH FIRST ROW ONLY"),
        [(1, 32, Literal(1)), (2, 1, Literal(1))]
    );
    // A written count is where the finding points.
    assert_eq!(
        limits("SELECT 1 FROM t\nFETCH FIRST 5 ROWS ONLY"),
        [(2, 13, Literal(5))]
    );
}

#[test]
fn with_ties_writes_a_count_but_is_not_a_page() {
    assert_eq!(
        values("SELECT id FROM t ORDER BY id FETCH FIRST 5 ROWS WITH TIES"),
        [SqlLimitValue::Literal(5)]
    );
    assert!(
        sweeps("SELECT id FROM t WHERE id > $1 ORDER BY id FETCH FIRST 5 ROWS WITH TIES")
            .is_empty()
    );
}

#[test]
fn a_sweep_records_the_column_of_its_table() {
    let found = sweeps("SELECT id\nFROM orders ORDER BY id LIMIT 3");
    assert_eq!((found[0].line, found[0].column), (2, 6));
}

#[test]
fn an_output_alias_in_order_by_means_the_aliased_expression() {
    // PostgreSQL resolves a bare ORDER BY name to the output column first.
    assert!(sweeps("SELECT random() AS id FROM orders ORDER BY id LIMIT $1").is_empty());
    assert_eq!(
        shape("SELECT id AS id FROM t ORDER BY id LIMIT 3"),
        ["t id | "]
    );
    assert_eq!(
        shape("SELECT created AS id FROM t ORDER BY id LIMIT 3"),
        ["t created | "]
    );
    // A qualified name is never an alias.
    assert_eq!(
        shape("SELECT random() AS id FROM t ORDER BY t.id LIMIT 3"),
        ["t id | "]
    );
}

#[test]
fn an_empty_page_is_a_cap_but_not_a_sweep() {
    assert!(sweeps("SELECT id FROM orders ORDER BY id LIMIT 0").is_empty());
    assert!(sweeps("SELECT id FROM orders ORDER BY id FETCH FIRST 0 ROWS ONLY").is_empty());
    assert_eq!(
        values("SELECT id FROM orders ORDER BY id LIMIT 0"),
        [SqlLimitValue::Literal(0)]
    );
}

#[test]
fn a_cursor_records_which_side_it_bounds() {
    use SqlCursorBound::{Lower, Upper};
    let bounds = |sql: &str| -> Vec<Option<SqlCursorBound>> {
        sweeps(sql)[0]
            .conjuncts
            .iter()
            .map(|conjunct| conjunct.cursor_bound)
            .collect()
    };
    assert_eq!(
        bounds("SELECT 1 FROM t WHERE id > $1 AND id <= $2 ORDER BY id LIMIT 3"),
        [Some(Lower), Some(Upper)]
    );
    assert_eq!(
        bounds("SELECT 1 FROM t WHERE $1 < id AND $2 > id ORDER BY id LIMIT 3"),
        [Some(Lower), Some(Upper)]
    );
    assert_eq!(
        bounds("SELECT 1 FROM t WHERE (a, b) < ($1, $2) ORDER BY a, b LIMIT 3"),
        [Some(Upper)]
    );
    assert_eq!(
        bounds("SELECT 1 FROM t WHERE ($1 IS NULL OR id >= $1) AND k = 1 ORDER BY id LIMIT 3"),
        [Some(Lower), None]
    );
}

#[test]
fn parentheses_around_the_select_do_not_hide_the_sweep() {
    assert_eq!(
        shape("(SELECT id FROM orders WHERE id > $1) ORDER BY id LIMIT $2"),
        ["orders id | id > $1(id)"]
    );
    // An inner query with modifiers of its own is a page by itself, and the outer one is not.
    assert_eq!(
        sweeps("(SELECT id FROM orders ORDER BY id LIMIT 5) ORDER BY id LIMIT 3").len(),
        1
    );
}

#[test]
fn a_quoted_dot_is_not_a_schema_qualifier() {
    assert!(sweeps(
        "WITH \"work.items\" AS (SELECT 1 AS id) SELECT id FROM \"work.items\" ORDER BY id LIMIT $1"
    )
    .is_empty());
    assert_eq!(
        sweeps("WITH t AS (SELECT 1) SELECT id FROM public.t ORDER BY id LIMIT 3").len(),
        1
    );
}

#[test]
fn constant_true_conjuncts_and_sampled_tables() {
    let constant = |sql: &str| -> Vec<bool> {
        sweeps(sql)[0]
            .conjuncts
            .iter()
            .map(|conjunct| conjunct.constant_true)
            .collect()
    };
    assert_eq!(
        constant(
            "SELECT 1 FROM t WHERE TRUE AND id > $1 AND (1 = 1) AND a = 1 ORDER BY id LIMIT 3"
        ),
        [true, false, true, false]
    );
    // Equal binds or NULLs are not constants, and neither is FALSE.
    assert_eq!(
        constant("SELECT 1 FROM t WHERE $1 = $1 AND NULL = NULL AND FALSE ORDER BY id LIMIT 3"),
        [false, false, false]
    );
    // A TABLESAMPLE restricts the relation before it is ordered.
    assert!(sweeps("SELECT id FROM t TABLESAMPLE SYSTEM (10) ORDER BY id LIMIT $1").is_empty());
}

#[test]
fn a_quoted_dot_in_a_qualifier_is_one_name() {
    assert_eq!(
        sweeps("SELECT id FROM \"work.items\" ORDER BY \"work.items\".id LIMIT $1").len(),
        1
    );
    assert_eq!(
        sweeps("SELECT id FROM public.orders o ORDER BY o.id LIMIT $1").len(),
        1
    );
}

#[test]
fn an_implicit_output_label_in_order_by_means_the_output_expression() {
    // PostgreSQL labels an unaliased `random()` as `random`.
    assert!(sweeps("SELECT random() FROM orders ORDER BY random LIMIT $1").is_empty());
    assert!(sweeps("SELECT random()::int FROM orders ORDER BY random LIMIT $1").is_empty());
    // A plain column keeps its own name.
    assert_eq!(
        shape("SELECT id, name FROM t ORDER BY name LIMIT 3"),
        ["t name | "]
    );
}

#[test]
fn an_optional_cursor_is_marked() {
    let optional = |sql: &str| -> Vec<bool> {
        sweeps(sql)[0]
            .conjuncts
            .iter()
            .map(|conjunct| conjunct.cursor_optional)
            .collect()
    };
    assert_eq!(
        optional("SELECT 1 FROM t WHERE ($1 IS NULL OR id >= $1) AND id < $2 AND k = 1 ORDER BY id LIMIT 3"),
        [true, false, false]
    );
}

#[test]
fn an_inner_order_by_orders_the_rows_an_outer_limit_counts() {
    assert_eq!(
        shape("(SELECT id FROM orders WHERE id > $1 ORDER BY id) LIMIT $2"),
        ["orders id | id > $1(id)"]
    );
    assert_eq!(
        shape("((SELECT id FROM orders ORDER BY id)) FETCH FIRST 5 ROWS ONLY"),
        ["orders id | "]
    );
    // The outermost ORDER BY is the order, whatever the inner one says.
    assert_eq!(
        shape("(SELECT id FROM orders ORDER BY name) ORDER BY id LIMIT 3"),
        ["orders id | "]
    );
    // No ordering anywhere is no walk, and a limit of its own makes the inner query its own page.
    assert!(sweeps("(SELECT id FROM orders) LIMIT 3").is_empty());
    assert_eq!(
        shape("(SELECT id FROM orders ORDER BY id LIMIT 5) LIMIT 3"),
        ["orders id | "]
    );
}

#[test]
fn a_sweep_records_the_parts_of_its_table_name() {
    let parts = |sql: &str| sweeps(sql)[0].table_parts.clone();
    assert_eq!(
        parts("SELECT id FROM work.items ORDER BY id LIMIT $1"),
        ["work", "items"]
    );
    assert_eq!(
        parts("SELECT id FROM \"work.items\" ORDER BY id LIMIT $1"),
        ["work.items"]
    );
    assert_eq!(
        parts("SELECT id FROM \"Work\".Items ORDER BY id LIMIT $1"),
        ["Work", "items"]
    );
}

#[test]
fn bind_only_guards_do_not_select_rows() {
    let facts = sweeps("SELECT id FROM orders WHERE $1::boolean IS NOT NULL AND id > $2 AND deleted_at IS NULL AND FALSE AND now() > $3 ORDER BY id LIMIT $4");
    assert_eq!(
        facts[0]
            .conjuncts
            .iter()
            .map(|fact| fact.bind_guard)
            .collect::<Vec<_>>(),
        vec![true, false, false, false, false]
    );
}

#[test]
fn distinct_pages_are_not_row_sweeps() {
    assert!(sweeps(
        "SELECT DISTINCT account_id FROM orders WHERE account_id > $1 ORDER BY account_id LIMIT $2"
    )
    .is_empty());
    assert!(sweeps("SELECT DISTINCT ON (account_id) account_id FROM orders WHERE account_id > $1 ORDER BY account_id LIMIT $2").is_empty());
}

#[test]
fn expanded_lexicographic_cursors() {
    for (predicate, expected) in [
        ("a > $1 OR (a = $1 AND b > $2)", vec!["a", "b"]),
        (
            "a < $1 OR (a = $1 AND b < $2) OR (a = $1 AND b = $2 AND c < $3)",
            vec!["a", "b", "c"],
        ),
        ("a > $1 OR (a = $1 AND b < $2)", vec![]),
        ("a > $1 OR (a = $3 AND b > $2)", vec![]),
        ("a > $1 OR (a = $1 AND lower(b) > $2)", vec![]),
        ("a > $1 OR b > $2", vec![]),
        ("a > $1 OR (a > $1 AND b > $2)", vec![]),
        ("a > $1 OR (c = $1 AND b > $2)", vec![]),
        ("a > $1 OR (a = $1 AND b >= $2)", vec![]),
        ("a > $1 OR (a = $1 AND b > 2)", vec![]),
        ("a > $1 OR (TRUE AND b > $2)", vec![]),
        ("a > $1 OR (a = $1 AND TRUE)", vec![]),
    ] {
        let sql = format!("SELECT a FROM t WHERE {predicate} ORDER BY a,b,c LIMIT $4");
        assert_eq!(
            sweeps(&sql)[0].conjuncts[0].cursor_columns,
            expected,
            "{predicate}"
        );
    }
}

#[test]
fn sweep_table_parts_preserve_postgres_case() {
    for (name, expected) in [
        ("\"Orders\"", "Orders"),
        ("Orders", "orders"),
        ("orders", "orders"),
    ] {
        assert_eq!(
            sweeps(&format!("SELECT id FROM {name} ORDER BY id LIMIT $1"))[0].table_parts,
            vec![expected]
        );
    }
}
