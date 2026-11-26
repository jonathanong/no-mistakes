use super::*;
use crate::codebase::postgres::statements::{
    extract_sql_statement_facts_with_policy, SqlDuplicateKeyedLookupFact, SqlLocationDemand,
    StatementPolicySources,
};
use crate::codebase::rules::validate_options::validate_rule_options;
use crate::config::v2::NoMistakesConfig;

fn collect(sql: &str) -> Vec<SqlDuplicateKeyedLookupFact> {
    let policy = StatementPolicySources {
        collect_duplicate_keyed_lookups: true,
        ..Default::default()
    };
    extract_sql_statement_facts_with_policy(
        sql,
        false,
        None,
        SqlLocationDemand::None,
        policy,
        false,
    )
    .duplicate_keyed_lookups
}

#[test]
fn detects_duplicate_scalar_subqueries_on_same_key() {
    let facts = collect(
        "SELECT
            o.id,
            (SELECT display_name FROM accounts WHERE id = o.account_id) AS name,
            (SELECT tier FROM accounts WHERE id = o.account_id) AS tier
        FROM orders o",
    );
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].relation, "accounts");
    assert_eq!(facts[0].first_line, 3);
    assert_eq!(facts[0].line, 4);
}

#[test]
fn detects_duplicate_joins_with_different_aliases_and_reversed_operands() {
    let facts = collect(
        "SELECT o.id, a1.display_name, a2.tier
        FROM orders o
        LEFT JOIN accounts a1 ON a1.id = o.account_id
        LEFT JOIN accounts a2 ON o.account_id = a2.id",
    );
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].relation, "accounts");
    assert_eq!(facts[0].first_line, 3);
    assert_eq!(facts[0].line, 4);
}

#[test]
fn detects_join_plus_scalar_subquery_and_equivalent_casts() {
    let facts = collect(
        "SELECT
            o.id,
            p.display_name,
            (SELECT p2.avatar_url FROM profiles p2 WHERE p2.id = CAST(o.owner_id AS uuid))
        FROM orders o
        LEFT JOIN profiles p ON p.id = o.owner_id::uuid",
    );
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].relation, "profiles");
}

#[test]
fn detects_join_using_duplicate() {
    let facts = collect(
        "SELECT o.id, a1.name, a2.tier
        FROM orders o
        LEFT JOIN accounts a1 USING (account_id)
        LEFT JOIN accounts a2 ON a2.account_id = o.account_id",
    );
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].relation, "accounts");
}

#[test]
fn ignores_distinct_keys_casts_and_residual_filters() {
    let sql = "
        SELECT o.id, b.name, s.name
        FROM orders o
        LEFT JOIN accounts b ON b.id = o.buyer_id
        LEFT JOIN accounts s ON s.id = o.seller_id;

        SELECT o.id, a1.name, a2.tier
        FROM orders o
        LEFT JOIN accounts a1 ON a1.id = o.raw_id::uuid
        LEFT JOIN accounts a2 ON a2.id = o.raw_id::text;

        SELECT o.id, b.postal_code, s.postal_code
        FROM orders o
        LEFT JOIN addresses b ON b.account_id = o.account_id AND b.kind = 'billing'
        LEFT JOIN addresses s ON s.account_id = o.account_id AND s.kind = 'shipping';
    ";
    assert!(collect(sql).is_empty());
}

#[test]
fn ignores_correlated_aggregates_exists_case_arms_and_volatile_keys() {
    let sql = "
        SELECT
            o.id,
            a.display_name,
            (SELECT COUNT(*) FROM accounts a2 WHERE a2.id = o.account_id),
            (SELECT SUM(a3.credits) FROM accounts a3 WHERE a3.id = o.account_id)
        FROM orders o
        LEFT JOIN accounts a ON a.id = o.account_id
        WHERE EXISTS (SELECT 1 FROM accounts a4 WHERE a4.id = o.account_id);

        SELECT
            o.id,
            CASE
                WHEN o.flag THEN (SELECT a1.short_name FROM accounts a1 WHERE a1.id = o.account_id)
                ELSE (SELECT a2.display_name FROM accounts a2 WHERE a2.id = o.account_id)
            END
        FROM orders o;

        SELECT o.id, a1.name, a2.tier
        FROM orders o
        LEFT JOIN accounts a1 ON a1.id = gen_random_uuid()
        LEFT JOIN accounts a2 ON a2.id = gen_random_uuid();
    ";
    assert!(collect(sql).is_empty());
}

#[test]
fn detects_duplicate_lookups_in_cte_update_and_delete() {
    let sql = "
        WITH enriched AS (
            SELECT
                o.id,
                (SELECT a1.name FROM accounts a1 WHERE a1.id = o.account_id) AS n,
                (SELECT a2.tier FROM accounts a2 WHERE a2.id = o.account_id) AS t
            FROM orders o
        )
        SELECT id FROM enriched;

        UPDATE orders o
        SET
            cached_name = (SELECT a1.name FROM accounts a1 WHERE a1.id = o.account_id),
            cached_tier = (SELECT a2.tier FROM accounts a2 WHERE a2.id = o.account_id);

        DELETE FROM orders o
        WHERE (SELECT a1.active FROM accounts a1 WHERE a1.id = o.account_id)
          AND (SELECT a2.locked FROM accounts a2 WHERE a2.id = o.account_id);
    ";
    let facts = collect(sql);
    assert_eq!(facts.len(), 3);
}

#[test]
fn detects_duplicate_lookups_in_with_dml_modifying_ctes_and_merge() {
    let sql = "
        WITH target_orders AS (SELECT id FROM orders WHERE status = 'pending')
        UPDATE orders o
        SET
            cached_name = (SELECT a1.name FROM accounts a1 WHERE a1.id = o.account_id),
            cached_tier = (SELECT a2.tier FROM accounts a2 WHERE a2.id = o.account_id)
        FROM target_orders t
        WHERE o.id = t.id;

        WITH removed AS (
            DELETE FROM orders o
            WHERE (SELECT a1.suspended FROM accounts a1 WHERE a1.id = o.account_id)
              AND (SELECT a2.archived FROM accounts a2 WHERE a2.id = o.account_id)
            RETURNING o.id
        )
        SELECT id FROM removed;

        MERGE INTO orders o
        USING incoming_orders s
        ON o.id = s.id
        WHEN MATCHED THEN
            UPDATE SET
                cached_name = (SELECT a1.name FROM accounts a1 WHERE a1.id = s.account_id),
                cached_tier = (SELECT a2.tier FROM accounts a2 WHERE a2.id = s.account_id)
        WHEN NOT MATCHED THEN
            INSERT (id, cached_name)
            VALUES (s.id, (SELECT a3.name FROM accounts a3 WHERE a3.id = s.account_id));
    ";
    let facts = collect(sql);
    assert_eq!(facts.len(), 3);
}

#[test]
fn validates_invalid_options_early() {
    let configs = crate::test_support::rule_fixture_root(RULE_ID)
        .parent()
        .unwrap()
        .join("configs");
    for (file, expected) in [
        ("invalid-unknown-option.yml", "unknownFlag"),
        ("invalid-missing-executor.yml", "importSpecifier"),
        ("invalid-both-executors.yml", "extraExecutorNames"),
        ("invalid-glob.yml", "sqlInclude"),
        ("invalid-unanalyzable.yml", "unanalyzableSql"),
    ] {
        let path = configs.join(file);
        let config: NoMistakesConfig =
            serde_yaml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let err = validate_rule_options(&configs, &path, &config, &|_| None)
            .expect_err(file)
            .to_string();
        assert!(
            err.contains(expected),
            "{file}: expected error containing `{expected}`, got `{err}`"
        );
    }
}
