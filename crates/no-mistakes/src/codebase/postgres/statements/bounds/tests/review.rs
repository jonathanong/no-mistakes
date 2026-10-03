use super::{facts, shape};

/// `column@table+table` for each bare column the pins of the statement's target leave for the
/// catalog to place.
fn pin_reads(sql: &str) -> Vec<String> {
    let all = facts(sql);
    let fact = all.first().unwrap_or_else(|| panic!("no fact for {sql}"));
    fact.query.items[fact.target.unwrap_or(0)]
        .pins
        .iter()
        .flat_map(|pin| &pin.reads)
        .map(|read| format!("{}@{}", read.column, read.tables.join("+")))
        .collect()
}

#[test]
fn only_a_set_returning_built_in_over_given_arguments_is_sized_by_its_caller() {
    assert_eq!(
        shape("SELECT 1 FROM unnest($1::int[]) AS x"),
        ["select: other"]
    );
    assert_eq!(
        shape("SELECT 1 FROM generate_series(1, 10) g"),
        ["select: other"]
    );
    assert_eq!(
        shape("SELECT 1 FROM jsonb_array_elements($1::jsonb) AS e"),
        ["select: other"]
    );
    // Any other function can return rows from anywhere, and so can an array taken from a query.
    assert_eq!(
        shape("SELECT 1 FROM get_all_accounts() f"),
        ["select: opaque"]
    );
    assert_eq!(
        shape("SELECT 1 FROM unnest(ARRAY(SELECT id FROM t)) u"),
        ["select: opaque"]
    );
    // A VALUES list is its own text.
    assert_eq!(
        shape("SELECT 1 FROM (VALUES (1), (2)) v"),
        ["select: (other)"]
    );
}

#[test]
fn a_recursive_reference_bounds_nothing() {
    assert_eq!(
        shape(
            "WITH RECURSIVE r AS (SELECT id FROM n WHERE id = $1 \
             UNION ALL SELECT n.id FROM n JOIN r ON n.parent = r.id) SELECT 1 FROM r"
        ),
        ["select: ((n[id=value]) (n[parent=#1] (opaque)))"]
    );
}

#[test]
fn an_aggregate_in_having_or_among_the_rare_built_ins_caps_a_query() {
    assert_eq!(
        shape("SELECT 1 FROM t HAVING count(*) > 0"),
        ["select: capped t"]
    );
    for call in [
        "corr(a, b)",
        "covar_pop(a, b)",
        "regr_slope(a, b)",
        "json_arrayagg(a)",
    ] {
        assert_eq!(
            shape(&format!("SELECT {call} FROM t")),
            ["select: capped t"],
            "{call}"
        );
    }
    // A set-returning function in the select list turns one aggregate row into many.
    assert_eq!(
        shape("SELECT generate_series(1, count(*)) FROM t"),
        ["select: t opaque"]
    );
    assert_eq!(
        shape("SELECT unnest(array_agg(id)) FROM t"),
        ["select: t opaque"]
    );
    // A window function is not an aggregate, whatever it is called.
    assert_eq!(shape("SELECT count(*) OVER () FROM t"), ["select: t"]);
}

#[test]
fn only_a_fixed_count_caps_a_query() {
    for limit in [
        "LIMIT 5",
        "LIMIT $1",
        "LIMIT $1::int",
        "LIMIT (5)",
        "LIMIT LEAST($1, 100)",
        "LIMIT COALESCE($1, 100)",
        "LIMIT 2 * $1",
        "LIMIT sql_placeholder_1",
        "FETCH FIRST ROW ONLY",
        "FETCH FIRST $1 ROWS ONLY",
    ] {
        assert_eq!(
            shape(&format!("SELECT 1 FROM t {limit}")),
            ["select: capped t"],
            "{limit}"
        );
    }
    for limit in [
        "LIMIT NULL",
        "LIMIT NULL::bigint",
        "LIMIT ALL",
        "LIMIT (SELECT count(*) FROM t)",
        "LIMIT (SELECT NULL::bigint)",
        "LIMIT COALESCE($1, NULL)",
        "LIMIT n",
    ] {
        assert_eq!(
            shape(&format!("SELECT 1 FROM t {limit}")),
            ["select: t"],
            "{limit}"
        );
    }
}

#[test]
fn a_query_inside_copy_is_judged_like_a_select() {
    assert_eq!(shape("COPY (SELECT 1 FROM t) TO STDOUT"), ["select: t"]);
    assert_eq!(
        shape("COPY (SELECT 1 FROM t LIMIT 5) TO STDOUT"),
        ["select: capped t"]
    );
    assert!(shape("COPY t TO STDOUT").is_empty());
}

#[test]
fn array_from_a_query_is_not_a_value() {
    assert_eq!(
        shape("DELETE FROM t WHERE id = ANY(ARRAY(SELECT id FROM t))"),
        ["delete: t"]
    );
    assert_eq!(
        shape("DELETE FROM t WHERE id = ANY(ARRAY[$1, $2])"),
        ["delete: t[id=value]"]
    );
}

#[test]
fn quoted_relations_keep_their_identity_and_unaliased_tables_answer_to_their_bare_name() {
    assert_eq!(
        shape("SELECT 1 FROM \"Accounts\""),
        ["select: \"Accounts\""]
    );
    assert_eq!(shape("SELECT 1 FROM Accounts"), ["select: accounts"]);
    assert_eq!(
        shape("SELECT 1 FROM public.\"Order Items\""),
        ["select: public.\"Order Items\""]
    );
    assert_eq!(
        shape("SELECT 1 FROM \"Accounts\" WHERE \"Accounts\".id = $1"),
        ["select: \"Accounts\"[id=value]"]
    );
    assert_eq!(
        shape("SELECT 1 FROM public.orders WHERE orders.id = $1"),
        ["select: public.orders[id=value]"]
    );
    // A one-part quoted name with a dot is not a schema qualifier: it is the CTE it names.
    assert_eq!(
        shape("WITH \"work.items\" AS (SELECT 1) SELECT 1 FROM \"work.items\""),
        ["select: ()"]
    );
}

#[test]
fn a_lateral_source_that_reads_earlier_items_is_marked() {
    let flag = |sql: &str| facts(sql)[0].query.items[1].lateral;
    assert!(flag(
        "SELECT 1 FROM a JOIN LATERAL (SELECT a.id AS id) d ON a.id = d.id"
    ));
    assert!(!flag("SELECT 1 FROM a, LATERAL (SELECT 1 AS id) d"));
    assert!(!flag("SELECT 1 FROM a, (SELECT 1 AS id) d"));
}

#[test]
fn correlation_is_resolved_one_query_level_at_a_time() {
    // A relation of a deeper level does not hide an outer reference at a shallower one.
    assert_eq!(
        shape("DELETE FROM t WHERE t.id IN (SELECT t.id FROM u WHERE u.x IN (SELECT x FROM t))"),
        ["delete: t"]
    );
    assert_eq!(
        shape("DELETE FROM t WHERE id IN (SELECT id WHERE EXISTS (SELECT 1 FROM u))"),
        ["delete: t"]
    );
    assert_eq!(
        shape(
            "DELETE FROM t a WHERE a.id IN (SELECT a.id WHERE EXISTS (SELECT 1 FROM u a LIMIT 1))"
        ),
        ["delete: t"]
    );
    // Nothing outer is read: the subquery sizes the key.
    assert_eq!(
        shape("DELETE FROM t WHERE id IN (SELECT id FROM u WHERE u.k IN (SELECT k FROM t))"),
        ["delete: t[id=(u[k=(t)])]"]
    );
}

#[test]
fn a_table_function_over_columns_or_of_a_user_schema_is_opaque() {
    // A column argument makes the factor lateral: it is evaluated per row of that item.
    assert_eq!(
        shape("SELECT 1 FROM a JOIN unnest(ARRAY[a.id]) u(id) ON a.id = u.id"),
        ["select: a[id=#1] opaque"]
    );
    assert_eq!(
        shape("SELECT 1 FROM a, generate_series(1, a.n) g"),
        ["select: a opaque"]
    );
    // Only a bare or pg_catalog name is the built-in.
    assert_eq!(
        shape("SELECT 1 FROM generate_series(1, 10) g"),
        ["select: other"]
    );
    assert_eq!(
        shape("SELECT 1 FROM pg_catalog.generate_series(1, 10) g"),
        ["select: other"]
    );
    assert_eq!(
        shape("SELECT 1 FROM app.generate_series(1, 10) g"),
        ["select: opaque"]
    );
}

#[test]
fn a_function_that_can_return_null_does_not_fix_a_count() {
    assert_eq!(shape("SELECT 1 FROM t LIMIT NULLIF(1, 1)"), ["select: t"]);
    assert_eq!(shape("SELECT 1 FROM t LIMIT abs($1)"), ["select: t"]);
    assert_eq!(
        shape("SELECT 1 FROM t LIMIT GREATEST($1, 5)"),
        ["select: capped t"]
    );
}

#[test]
fn an_aggregate_only_in_order_by_makes_one_row() {
    assert_eq!(
        shape("SELECT 1 FROM t ORDER BY count(*)"),
        ["select: capped t"]
    );
    assert_eq!(shape("SELECT 1 FROM t ORDER BY id"), ["select: t"]);
    assert_eq!(
        shape("SELECT 1 FROM t GROUP BY k ORDER BY count(*)"),
        ["select: t"]
    );
}

#[test]
fn a_chain_of_ctes_that_double_in_size_stays_small() {
    // Each CTE reads the one before it twice; without a cap the bound doubles at every step.
    let mut sql = String::from("WITH c0 AS (SELECT id FROM t)");
    for level in 1..40 {
        sql.push_str(&format!(
            ", c{level} AS (SELECT id FROM c{} UNION ALL SELECT id FROM c{})",
            level - 1,
            level - 1
        ));
    }
    sql.push_str(" SELECT 1 FROM c39");
    assert_eq!(facts(&sql).len(), 1);
}

#[test]
fn a_bare_column_is_kept_with_the_tables_that_could_own_it() {
    // PostgreSQL resolves `id` outward when none of the subquery's tables has it, which only the
    // catalog can tell.
    assert_eq!(
        pin_reads("DELETE FROM accounts a WHERE a.id IN (SELECT id FROM currencies LIMIT 1)"),
        ["id@currencies"]
    );
    // Every table of the level is a candidate, and so is every table of a level in between.
    assert_eq!(
        pin_reads(
            "DELETE FROM accounts a WHERE a.id IN (SELECT 1 FROM currencies c JOIN rates r \
             ON c.code = r.code WHERE EXISTS (SELECT 1 FROM fees WHERE id = 1))"
        ),
        ["id@currencies+fees+rates"]
    );
    // A scalar subquery in the value, and in an IN list, is read the same way.
    assert_eq!(
        pin_reads("DELETE FROM accounts WHERE id = (SELECT id FROM currencies LIMIT 1)"),
        ["id@currencies"]
    );
    assert_eq!(
        pin_reads(
            "DELETE FROM accounts WHERE id IN ($1, (SELECT id FROM currencies LIMIT 1), \
             (SELECT id FROM currencies LIMIT 1))"
        ),
        ["id@currencies"]
    );
}

#[test]
fn a_bare_column_a_relation_other_than_a_base_table_may_own_is_not_an_outer_read() {
    for sql in [
        "DELETE FROM accounts WHERE id IN (SELECT id FROM (SELECT id FROM currencies) c LIMIT 1)",
        "DELETE FROM accounts WHERE id IN (SELECT id FROM generate_series(1, 3) id LIMIT 1)",
        "DELETE FROM accounts WHERE id IN (SELECT id FROM currencies, LATERAL (SELECT 1) l LIMIT 1)",
        "WITH c AS (SELECT 1 AS id) DELETE FROM accounts WHERE id IN (SELECT id FROM c LIMIT 1)",
        // `c` is a CTE of the subquery, not the base table of that name.
        "DELETE FROM accounts WHERE id IN (WITH c AS (SELECT 1 AS id) SELECT id FROM c LIMIT 1)",
    ] {
        assert_eq!(pin_reads(sql), Vec::<String>::new(), "{sql}");
    }
}

#[test]
fn a_subquery_with_no_relation_of_its_own_reads_the_row_for_certain() {
    // No pin at all: every row finds itself.
    assert_eq!(
        shape("DELETE FROM accounts WHERE id IN (SELECT id)"),
        ["delete: accounts"]
    );
    assert_eq!(
        shape("DELETE FROM accounts WHERE id = (SELECT id)"),
        ["delete: accounts"]
    );
}

#[test]
fn output_names_whole_rows_and_keywords_are_not_columns() {
    // `n` names an output column and `s` is the whole row of `sessions`: only `user_id` and
    // `owner` are read as columns. `current_user` is not a column.
    assert_eq!(
        pin_reads(
            "DELETE FROM accounts WHERE id IN (SELECT user_id, count(*) AS n FROM sessions s \
             WHERE s IS NOT NULL AND owner = current_user GROUP BY user_id ORDER BY n LIMIT 5)"
        ),
        ["owner@sessions", "user_id@sessions"]
    );
    // The same name used both as an output and as a column is still read as a column.
    assert_eq!(
        pin_reads(
            "DELETE FROM accounts WHERE id IN (SELECT count(*) AS n FROM sessions \
             WHERE n > 1 ORDER BY n LIMIT 5)"
        ),
        ["n@sessions"]
    );
}

#[test]
fn a_lateral_source_keeps_its_bare_reads_for_the_catalog() {
    let reads = |sql: &str| {
        let item = &facts(sql)[0].query.items[1];
        (
            item.lateral,
            item.lateral_reads
                .iter()
                .map(|read| format!("{}@{}", read.column, read.tables.join("+")))
                .collect::<Vec<_>>(),
        )
    };
    assert_eq!(
        reads("SELECT 1 FROM accounts a, LATERAL (SELECT id FROM currencies LIMIT 1) c"),
        (false, vec!["id@currencies".to_string()])
    );
    assert_eq!(
        reads("SELECT 1 FROM accounts a, LATERAL (SELECT a.id FROM currencies LIMIT 1) c"),
        (true, Vec::new())
    );
    assert_eq!(
        reads("SELECT 1 FROM accounts a, (SELECT id FROM currencies LIMIT 1) c"),
        (false, Vec::new())
    );
}

#[test]
fn a_table_arm_names_a_cte_when_one_has_that_name() {
    assert_eq!(
        shape("WITH c AS (SELECT * FROM orders) SELECT 1 UNION ALL TABLE c"),
        ["select: () ((orders))"]
    );
    // A schema-qualified name is the table, whatever the CTE is called.
    assert_eq!(
        shape("WITH c AS (SELECT * FROM orders) SELECT 1 UNION ALL TABLE public.c"),
        ["select: () (public.c)"]
    );
}

#[test]
fn an_explicit_collation_in_the_value_fixes_no_row() {
    assert_eq!(
        shape("SELECT 1 FROM accounts WHERE email = $1 COLLATE \"C\""),
        ["select: accounts"]
    );
    assert_eq!(
        shape("SELECT 1 FROM accounts WHERE email IN ($1, ($2 COLLATE \"C\"))"),
        ["select: accounts"]
    );
    assert_eq!(
        shape("SELECT 1 FROM accounts WHERE email = $1::text"),
        ["select: accounts[email=value]"]
    );
}

#[test]
fn oversized_ctes_retain_uncapped_relation_reads() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/oversized-cte.sql"
    ));
    let bound = facts(sql);
    assert_eq!(bound.len(), 2);
    assert!(shape(sql)[0].contains("orders"));
    assert!(bound[1].query.capped);
}

#[test]
fn oversized_cte_compaction_keeps_only_pins_common_to_every_relation_read() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/oversized-cte-pins.sql"
    ));
    assert_eq!(shape(sql).len(), 2);
    assert!(shape(sql)[0].contains("orders[id=value]"));
    assert!(!shape(sql)[1].contains("orders[id=value]"));
}

#[test]
fn every_known_set_returning_builtin_expands_aggregate_rows() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/set-returning-builtins.sql"));
    let shapes = shape(sql);
    // Comments in this parser-only inventory are not statements in the shape result.
    let fixed_one_row = sql
        .lines()
        .filter(|line| line.starts_with("SELECT "))
        .position(|line| line.starts_with("SELECT pg_stat_get_recovery_prefetch("))
        .unwrap();
    assert!(shapes[..shapes.len() - 1]
        .iter()
        .enumerate()
        .all(|(index, shape)| {
            if index == fixed_one_row {
                shape == "select: capped orders"
            } else {
                shape == "select: orders opaque"
            }
        }));
    assert_eq!(shapes.last().unwrap(), "select: other");
}

#[test]
fn caller_sized_set_returning_functions_reject_server_derived_arguments() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/set-returning-arguments.sql"
    ));
    assert_eq!(
        shape(sql),
        [
            "select: opaque accounts[email=#0]",
            "select: opaque accounts[email=#0]",
            "select: opaque accounts[email=#0]",
            "select: opaque accounts[email=#0]",
            "select: other accounts[email=#0]",
        ]
    );
}

#[test]
fn server_state_set_returning_functions_stay_opaque_in_from() {
    for call in [
        "pg_ls_dir('/tmp')",
        "pg_listening_channels()",
        "ts_stat($1)",
    ] {
        assert_eq!(shape(&format!("SELECT 1 FROM {call}")), ["select: opaque"]);
    }
}

#[test]
fn catalog_set_returning_names_require_builtin_schema_identity() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/set-returning-qualified.sql"));
    assert_eq!(
        shape(sql),
        [
            "select: capped orders opaque",
            "select: orders opaque",
            "select: orders opaque"
        ]
    );
}

mod followups;
