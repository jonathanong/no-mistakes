use super::evaluate::offenders;
use super::{check_with_files, check_with_files_and_sources, compile_options, Options, RULE_ID};
use crate::codebase::postgres::{extract_sql_statement_facts, SchemaCatalog};
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

mod referential;
mod review;
mod suppression;
mod trigger_scope;

fn catalog() -> SchemaCatalog {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-bounded-statements/fixture");
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(
        &root.join("schema.json"),
    ));
    SchemaCatalog::load(&root, "schema.json", &sources).unwrap()
}

/// (table, line) for every relation that makes any statement in `sql` unbounded.
fn unbounded(sql: &str) -> Vec<(String, usize)> {
    let catalog = catalog();
    extract_sql_statement_facts(sql)
        .bounds
        .iter()
        .flat_map(|fact| offenders(fact, &catalog))
        .map(|offender| (offender.table, offender.line))
        .collect()
}

pub(super) fn names(sql: &str) -> Vec<String> {
    unbounded(sql).into_iter().map(|(table, _)| table).collect()
}

#[test]
fn a_whole_unique_key_pinned_to_a_value_bounds_a_relation() {
    assert!(names("SELECT 1 FROM accounts WHERE id = $1").is_empty());
    assert!(names("SELECT 1 FROM accounts WHERE email = lower($1)").is_empty());
    assert!(names("SELECT 1 FROM public.accounts WHERE id = ANY($1)").is_empty());
    assert_eq!(
        names("SELECT 1 FROM accounts WHERE name = $1"),
        ["accounts"]
    );
    assert_eq!(names("SELECT 1 FROM accounts"), ["accounts"]);
    assert_eq!(
        names("SELECT 1 FROM order_lines WHERE line_no = $1"),
        ["order_lines"]
    );
}

#[test]
fn a_limit_a_pure_aggregate_or_an_unknown_relation_needs_no_key() {
    assert!(names("SELECT 1 FROM orders LIMIT 10").is_empty());
    assert!(names("SELECT count(*) FROM orders").is_empty());
    assert!(names("SELECT 1 FROM mystery_table").is_empty());
    assert!(names("SELECT 1 FROM unnest($1::int[]) AS x").is_empty());
    assert!(names("SELECT 1").is_empty());
    assert_eq!(
        names("SELECT status, count(*) FROM orders GROUP BY status"),
        ["orders"]
    );
}

#[test]
fn bounded_items_bound_the_relations_pinned_to_them() {
    // A chain: o is pinned by value, a by o, and nothing else is needed.
    assert!(
        names("SELECT 1 FROM orders o, accounts a WHERE o.id = $1 AND a.id = o.account_id")
            .is_empty()
    );
    // Equality between two unbounded relations bounds neither.
    assert_eq!(
        names("SELECT 1 FROM orders o, accounts a WHERE a.id = o.id AND o.id = a.id"),
        ["orders", "accounts"]
    );
    // A relation joined to a capped CTE by key is bounded by the CTE's limit.
    assert!(names(
        "WITH c AS (SELECT id FROM orders ORDER BY id LIMIT $1) \
         SELECT 1 FROM c JOIN accounts a ON a.id = c.id"
    )
    .is_empty());
    // An uncapped CTE reports its own tables and bounds nothing joined to it.
    assert_eq!(
        names("WITH c AS (SELECT id FROM orders) SELECT 1 FROM c JOIN accounts a ON a.id = c.id"),
        ["orders", "accounts"]
    );
}

#[test]
fn set_operations_need_every_arm_bounded() {
    assert!(names(
        "SELECT id FROM accounts WHERE id = $1 UNION ALL SELECT id FROM orders WHERE id = $1"
    )
    .is_empty());
    assert_eq!(
        names("SELECT id FROM accounts WHERE id = $1 UNION ALL SELECT id FROM orders"),
        ["orders"]
    );
    assert!(names("SELECT id FROM accounts UNION ALL SELECT id FROM orders LIMIT 5").is_empty());
}

#[test]
fn a_derived_table_is_bounded_when_its_query_is() {
    assert!(names("SELECT * FROM (SELECT id FROM orders ORDER BY id LIMIT 3) d").is_empty());
    assert_eq!(names("SELECT * FROM (SELECT id FROM orders) d"), ["orders"]);
}

#[test]
fn updates_and_deletes_need_only_their_target_bounded() {
    assert!(names("UPDATE orders SET status = 'x' WHERE id = $1").is_empty());
    assert_eq!(
        names("UPDATE orders SET status = 'x' WHERE account_id = $1"),
        ["orders"]
    );
    // The USING relation may be unbounded: each target row changes once.
    assert!(
        names("DELETE FROM orders o USING accounts a WHERE o.id = $1 AND a.name = o.status")
            .is_empty()
    );
    assert_eq!(
        names("DELETE FROM orders WHERE id IN (SELECT id FROM orders)"),
        ["orders"]
    );
    assert!(names("DELETE FROM orders WHERE id IN (SELECT id FROM orders LIMIT 4)").is_empty());
    // `ctid` identifies one row of any table.
    assert!(names("DELETE FROM mystery WHERE ctid = $1").is_empty());
    assert!(names("DELETE FROM sessions WHERE ctid = ANY($1)").is_empty());
    assert!(names("DELETE FROM sessions WHERE expires_at < now() LIMIT 5").is_empty());
}

#[test]
fn a_relation_the_catalog_names_in_another_form_is_found() {
    assert_eq!(names("SELECT 1 FROM public.orders"), ["orders"]);
    assert_eq!(
        names("SELECT 1 FROM \"orders\" AS o WHERE o.status = $1"),
        ["orders"]
    );
}

#[test]
fn offenders_carry_the_line_of_each_relation() {
    assert_eq!(
        unbounded("SELECT 1\n  FROM orders o\n  JOIN accounts a ON a.name = o.status"),
        [("orders".to_string(), 2), ("accounts".to_string(), 3)]
    );
}

#[test]
fn quoted_key_columns_match_quoted_sql_columns() {
    let catalog = SchemaCatalog::from_json(
        r#"{
          "formatVersion": 2, "coverage": "complete",
          "tables": { "mixed": {
            "columns": { "Order Id": { "dataType": "uuid" } },
            "indexes": { "mixed_pkey": {
              "accessMethod": "btree",
              "unique": true, "primary": true, "valid": true, "ready": true,
              "keys": [ { "column": "Order Id", "expression": "\"Order Id\"" } ]
            } }
          } }
        }"#,
    )
    .unwrap();
    assert_eq!(catalog.unique_keys("mixed"), [vec!["Order Id".to_string()]]);
    let judge = |sql: &str| {
        extract_sql_statement_facts(sql)
            .bounds
            .iter()
            .flat_map(|fact| offenders(fact, &catalog))
            .count()
    };
    assert_eq!(judge("SELECT 1 FROM mixed WHERE \"Order Id\" = $1"), 0);
    assert_eq!(judge("SELECT 1 FROM mixed WHERE order_id = $1"), 1);
}

#[test]
fn an_unknown_relation_is_not_judged_and_bounds_nothing() {
    // Nothing says how many accounts an unknown relation names.
    assert_eq!(
        names(
            "UPDATE accounts a SET name = 'x' FROM external_accounts e WHERE a.id = e.account_id"
        ),
        ["accounts"]
    );
    assert_eq!(
        names("SELECT 1 FROM accounts a JOIN external e ON a.id = e.account_id WHERE e.id = $1"),
        ["accounts"]
    );
    assert_eq!(
        names("DELETE FROM accounts WHERE id IN (SELECT account_id FROM external_accounts)"),
        ["accounts"]
    );
    // It is never reported itself, and a limit on the subquery still bounds the key.
    assert!(names("SELECT 1 FROM external e WHERE e.id = $1").is_empty());
    assert!(names(
        "DELETE FROM accounts WHERE id IN (SELECT account_id FROM external_accounts LIMIT 5)"
    )
    .is_empty());
}

#[test]
fn a_subquery_that_reads_the_row_is_not_a_key_pin() {
    // The subquery yields each row's own id, so it matches every row whatever its bound.
    for sql in [
        "DELETE FROM accounts a WHERE a.id IN (SELECT a.id)",
        "DELETE FROM accounts WHERE id IN (SELECT id)",
        "DELETE FROM accounts a WHERE a.id = (SELECT a.id)",
        "DELETE FROM accounts a WHERE a.id = ANY (SELECT a.id)",
        "DELETE FROM accounts a WHERE a.id IN (SELECT o.account_id FROM orders o WHERE o.account_id = a.id LIMIT 5)",
    ] {
        assert_eq!(names(sql), ["accounts"], "{sql}");
    }
    // An independent subquery still sizes the key, and an inner relation shadows an outer name.
    for sql in [
        "DELETE FROM sessions WHERE id IN (SELECT id FROM sessions ORDER BY id LIMIT $1)",
        "DELETE FROM sessions s WHERE s.id IN (SELECT s.id FROM sessions s ORDER BY s.id LIMIT $1)",
        "DELETE FROM accounts WHERE id = (SELECT max(id) FROM accounts)",
    ] {
        assert!(names(sql).is_empty(), "{sql}");
    }
}

#[test]
fn a_function_that_differs_per_row_is_not_a_fixed_value() {
    for sql in [
        "DELETE FROM accounts WHERE id = nextval('account_ids')",
        "DELETE FROM accounts WHERE id = gen_random_uuid()",
        "DELETE FROM accounts WHERE id = pg_catalog.random()::text::uuid",
        "DELETE FROM accounts WHERE id = ANY(ARRAY[gen_random_uuid()])",
    ] {
        assert_eq!(names(sql), ["accounts"], "{sql}");
    }
    // Functions of a bind, and per-statement constants, are values.
    assert!(names("DELETE FROM accounts WHERE email = lower($1)").is_empty());
    assert!(names("DELETE FROM sessions WHERE id = now()::text::uuid").is_empty());
}

#[test]
fn table_returns_every_row_of_its_relation() {
    // The parser accepts TABLE only as an arm of a set operation.
    assert_eq!(
        names("SELECT id FROM accounts WHERE id = $1 UNION TABLE orders"),
        ["orders"]
    );
    assert_eq!(
        names("SELECT id FROM accounts WHERE id = $1 UNION TABLE public.accounts"),
        ["accounts"]
    );
    assert_eq!(
        names("SELECT id FROM accounts WHERE id = $1 UNION ALL TABLE public.orders LIMIT 5"),
        Vec::<String>::new()
    );
    assert!(names("SELECT id FROM accounts WHERE id = $1 UNION TABLE mystery").is_empty());
}

#[test]
fn a_dml_target_is_a_physical_relation_even_when_a_cte_shares_its_name() {
    assert_eq!(
        names("WITH accounts AS (SELECT 1) DELETE FROM accounts"),
        ["accounts"]
    );
    assert_eq!(
        names("WITH orders AS (SELECT 1 AS id) UPDATE orders SET status = 'x'"),
        ["orders"]
    );
    // A source item of the statement still resolves to the CTE.
    assert!(names(
        "WITH c AS (SELECT id FROM exports ORDER BY id LIMIT $1) \
         UPDATE exports SET s3_key = NULL FROM c WHERE exports.id = c.id"
    )
    .is_empty());
}

#[test]
fn a_relation_reached_by_several_arms_is_reported_once() {
    assert_eq!(
        names(
            "SELECT * FROM orders UNION ALL SELECT * FROM accounts UNION ALL SELECT * FROM orders"
        ),
        ["orders", "accounts"]
    );
}

#[test]
fn ctid_is_not_a_key_of_a_partitioned_table() {
    // The leaves of a partitioned table repeat ctid values.
    assert_eq!(names("DELETE FROM events WHERE ctid = $1"), ["events"]);
    assert_eq!(
        names(
            "UPDATE events SET id = id WHERE ctid IN (SELECT ctid FROM events ORDER BY id LIMIT 5)"
        ),
        ["events"]
    );
    assert!(names("DELETE FROM events WHERE id = $1 AND created_at = $2").is_empty());
    assert!(names("DELETE FROM sessions WHERE ctid = $1").is_empty());
}

#[test]
fn join_using_bounds_like_on() {
    assert!(names("SELECT 1 FROM accounts a JOIN orders o USING (id) WHERE a.id = $1").is_empty());
    assert!(
        names("SELECT 1 FROM accounts a LEFT JOIN orders o USING (id) WHERE a.id = $1").is_empty()
    );
    // The preserved side is not pinned by the join, and a longer left side is ambiguous.
    assert_eq!(
        names("SELECT 1 FROM accounts a LEFT JOIN orders o USING (id) WHERE o.status = $1"),
        ["accounts", "orders"]
    );
    assert_eq!(
        names("SELECT 1 FROM accounts a JOIN profiles p ON p.account_id = a.id JOIN orders o USING (id) WHERE a.id = $1"),
        ["orders"]
    );
}

#[test]
fn unique_keys_resolve_through_the_same_identity_as_the_table() {
    let root = fixture_root();
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(
        &root.join("schema-qualified.json"),
    ));
    let catalog = SchemaCatalog::load(&root, "schema-qualified.json", &sources).unwrap();
    assert_eq!(catalog.unique_keys("widgets"), [vec!["id".to_string()]]);
    let judged = |sql: &str| {
        extract_sql_statement_facts(sql)
            .bounds
            .iter()
            .map(|fact| offenders(fact, &catalog).len())
            .sum::<usize>()
    };
    assert_eq!(judged("SELECT * FROM widgets WHERE id = $1"), 0);
    assert_eq!(judged("SELECT * FROM widgets"), 1);
}

#[test]
fn a_null_safe_comparison_bounds_only_a_not_null_key() {
    // contacts.email is a nullable unique column: `$1` = NULL matches every row without one.
    assert_eq!(
        names("DELETE FROM contacts WHERE email IS NOT DISTINCT FROM $1"),
        ["contacts"]
    );
    assert!(names("DELETE FROM contacts WHERE email = $1").is_empty());
    // The primary key and ctid cannot be NULL.
    assert!(names("DELETE FROM contacts WHERE id IS NOT DISTINCT FROM $1").is_empty());
    assert!(names("DELETE FROM contacts WHERE $1 IS NOT DISTINCT FROM ctid").is_empty());
}

#[test]
fn a_recovered_interpolation_pins_like_a_placeholder() {
    // A `${…}` interpolation reaches the rule as `sql_placeholder_N`.
    for sql in [
        "SELECT id FROM invoices WHERE id = sql_placeholder_1",
        "SELECT id FROM invoices WHERE sql_placeholder_1 = id",
        "SELECT id FROM invoices WHERE id = sql_placeholder_1::uuid",
        "SELECT 1 FROM order_lines WHERE order_id = sql_placeholder_1 AND line_no = sql_placeholder_2",
        "UPDATE exports SET s3_key = NULL WHERE id = sql_placeholder_1 AND expires_at IS NOT NULL",
        "SELECT id FROM orders WHERE id IN (sql_placeholder_1, sql_placeholder_2)",
        "SELECT id FROM orders WHERE id = ANY(sql_placeholder_1::uuid[])",
        "SELECT o.id FROM orders o JOIN accounts a ON a.id = o.account_id WHERE o.id = sql_placeholder_1",
    ] {
        assert!(names(sql).is_empty(), "{sql}");
    }
    // Only a key is pinned: a bind compared with another column is not.
    assert_eq!(
        names("SELECT 1 FROM invoices WHERE paid_at = sql_placeholder_1"),
        ["invoices"]
    );
}

#[test]
fn a_bound_propagates_across_joins_on_unique_keys() {
    // One hop, whichever side of the join and of the equality the key is on.
    for sql in [
        "SELECT 1 FROM accounts a JOIN profiles p ON p.account_id = a.id WHERE a.id = $1",
        "SELECT 1 FROM accounts a JOIN profiles p ON a.id = p.account_id WHERE a.id = $1",
        "SELECT 1 FROM profiles p JOIN accounts a ON a.id = p.account_id WHERE a.id = $1",
        "SELECT 1 FROM accounts a JOIN profiles p ON p.account_id = a.id WHERE a.id = sql_placeholder_1",
    ] {
        assert!(names(sql).is_empty(), "{sql}");
    }
    // Two hops, as explicit joins and as a comma list with the conditions in WHERE.
    for sql in [
        "SELECT 1 FROM orders o JOIN accounts a ON a.id = o.account_id JOIN profiles p ON p.account_id = a.id WHERE o.id = $1",
        "SELECT 1 FROM orders o, accounts a, profiles p WHERE o.id = $1 AND a.id = o.account_id AND p.account_id = a.id",
    ] {
        assert!(names(sql).is_empty(), "{sql}");
    }
    // A hop through a column that is not a whole key ends the chain.
    assert_eq!(
        names("SELECT 1 FROM accounts a JOIN orders o ON o.account_id = a.id WHERE a.id = $1"),
        ["orders"]
    );
    assert_eq!(
        names("SELECT 1 FROM accounts a JOIN orders o ON o.account_id = a.id JOIN order_lines l ON l.order_id = o.id WHERE a.id = $1"),
        ["orders", "order_lines"]
    );
}

#[test]
fn outer_joins_bound_only_the_side_they_cannot_null_extend() {
    // The right side of a LEFT JOIN hangs off the left row by its key, so it is bounded with it.
    assert!(names(
        "SELECT 1 FROM accounts a LEFT JOIN profiles p ON p.account_id = a.id WHERE a.id = $1"
    )
    .is_empty());
    assert!(names(
        "SELECT 1 FROM profiles p RIGHT JOIN accounts a ON p.account_id = a.id WHERE a.id = $1"
    )
    .is_empty());
    // ON never restricts the preserved side: every account survives the join.
    assert_eq!(
        names("SELECT 1 FROM accounts a LEFT JOIN profiles p ON a.id = p.account_id AND p.account_id = $1"),
        ["accounts"]
    );
    // A WHERE on the null-extended side would make the join inner; that is not assumed.
    assert_eq!(
        names("SELECT 1 FROM accounts a LEFT JOIN profiles p ON p.account_id = a.id WHERE p.account_id = $1"),
        ["accounts"]
    );
}

#[test]
fn fetch_with_ties_and_foreign_aggregates_do_not_cap() {
    assert!(names("SELECT 1 FROM orders ORDER BY id FETCH FIRST 10 ROWS ONLY").is_empty());
    assert_eq!(
        names("SELECT 1 FROM orders ORDER BY status FETCH FIRST 10 ROWS WITH TIES"),
        ["orders"]
    );
    assert!(names("SELECT pg_catalog.count(*) FROM orders").is_empty());
    assert_eq!(names("SELECT app.count(id) FROM orders"), ["orders"]);
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-bounded-statements/fixture")
}

fn config(yaml: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(yaml).unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

#[test]
fn the_standalone_entry_points_prepare_their_own_facts() {
    let root = fixture_root();
    let files = [root.join("sql/invalid.sql"), root.join("schema.json")];
    let yaml =
        "schemaCatalogPath: schema.json\nsqlInclude: ['sql/invalid.sql']\nexclude: ['src/**']";
    let direct = check_with_files(&root, &config(yaml), &files).unwrap();
    assert_eq!(direct.len(), 3, "{direct:#?}");
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let shared = check_with_files_and_sources(&root, &config(yaml), &files, &sources).unwrap();
    assert_eq!(direct, shared);
}

#[test]
fn sql_that_cannot_be_analyzed_fails_closed_unless_ignored() {
    let root = fixture_root();
    let files = [
        root.join("sql/broken.sql"),
        root.join("src/dynamic.ts"),
        root.join("schema.json"),
    ];
    let base = "schemaCatalogPath: schema.json\nimportSpecifier: '@example/db'\nexecutorNames: [query, read, write]\nsqlInclude: ['sql/broken.sql']\n";
    let found: Vec<_> = check_with_files(&root, &config(base), &files)
        .unwrap()
        .into_iter()
        .map(|finding| (finding.file, finding.message, finding.target.unwrap()))
        .collect();
    assert_eq!(found.len(), 2, "{found:#?}");
    assert!(found.iter().all(|(_, _, target)| target == "unanalyzable"));
    assert!(found[0]
        .1
        .contains("SQL could not be analyzed for bounded statements"));
    assert!(found[1]
        .1
        .contains("not statically recoverable for bounded statements"));
    let ignored = format!("{base}unanalyzableSql: ignore\n");
    assert!(check_with_files(&root, &config(&ignored), &files)
        .unwrap()
        .is_empty());
}

#[test]
fn scanning_requires_prepared_facts() {
    let options: Options = serde_yaml::from_str("schemaCatalogPath: schema.json").unwrap();
    let compiled = compile_options(&options).unwrap();
    let sources = crate::codebase::rules::source_store_for_files(&[]);
    let error = super::scan::scan(Path::new("."), &compiled, &[], &sources, None)
        .err()
        .unwrap();
    assert!(error
        .to_string()
        .contains("prepared PostgreSQL facts are required"));
}
