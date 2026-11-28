use super::fixture_facts;

#[test]
fn reports_only_reachable_row_lookups_and_distinct_frame_outputs() {
    let facts = fixture_facts("review-row-guards");
    let sites: Vec<_> = facts
        .duplicate_keyed_lookups
        .iter()
        .map(|fact| (fact.first_line, fact.line))
        .collect();

    assert_eq!(
        sites,
        [
            (12, 13),
            (20, 21),
            (27, 28),
            (32, 33),
            (48, 49),
            (54, 55),
            (60, 61),
            (66, 67),
            (86, 87),
            (97, 96),
            (108, 109),
            (120, 121),
            (131, 132),
            (142, 143),
            (148, 149),
            (169, 170),
            (175, 176),
            (184, 185),
            (200, 201),
            (206, 207),
            (212, 213),
            (218, 219),
            (224, 225),
            (230, 231),
            (241, 242),
        ]
    );
}

#[test]
fn respects_planner_demand_and_equivalent_set_targets() {
    let facts = fixture_facts("review-planner-guards");
    let sites: Vec<_> = facts
        .duplicate_keyed_lookups
        .iter()
        .map(|fact| (fact.first_line, fact.line))
        .collect();

    assert_eq!(
        sites,
        [
            (15, 16),
            (23, 24),
            (35, 36),
            (41, 42),
            (67, 68),
            (79, 80),
            (101, 104),
            (111, 113),
            (122, 124),
            (139, 140),
            (147, 148),
            (159, 161),
            (166, 167),
            (173, 174),
        ]
    );
}

#[test]
fn respects_output_stages_and_row_demand() {
    let facts = fixture_facts("review-output-stages");
    let sites: Vec<_> = facts
        .duplicate_keyed_lookups
        .iter()
        .map(|fact| (fact.first_line, fact.line))
        .collect();

    assert_eq!(
        sites,
        [
            (7, 8),
            (18, 19),
            (24, 25),
            (40, 41),
            (50, 52),
            (61, 62),
            (66, 67),
            (77, 78),
            (83, 84),
            (92, 93),
            (97, 98),
            (106, 107),
            (111, 112),
        ]
    );
}

#[test]
fn preserves_returning_join_coalesce_cte_and_decimal_limit_guards() {
    let facts = fixture_facts("review-guard-propagation");
    let sites: Vec<_> = facts
        .duplicate_keyed_lookups
        .iter()
        .map(|fact| (fact.first_line, fact.line))
        .collect();

    assert_eq!(
        sites,
        [
            (9, 10),
            (15, 16),
            (21, 22),
            (35, 36),
            (42, 43),
            (54, 55),
            (61, 62),
            (68, 69),
            (75, 76),
            (82, 83),
            (102, 103),
            (112, 113),
            (130, 131),
            (139, 140),
            (178, 179),
            (186, 187),
            (194, 195),
        ]
    );
}

#[test]
fn covers_edge_sql_normalizer_and_clause_paths() {
    use crate::statements::{
        extract_sql_statement_facts_with_policy, SqlLocationDemand, StatementPolicySources,
    };

    let statements = [
        "SELECT EXISTS (SELECT (SELECT a.n FROM accounts a WHERE a.id = o.id) FROM orders o LIMIT 1e-2);",
        "SELECT EXISTS (SELECT (SELECT a.n FROM accounts a WHERE a.id = o.id) FROM orders o LIMIT 0x10);",
        "SELECT EXISTS (SELECT (SELECT a.n FROM accounts a WHERE a.id = o.id) FROM orders o LIMIT 1e999999999999999999999);",
        "SELECT o.id FROM orders o
        LEFT JOIN accounts a1 ON a1.id = o.account_id
            AND (pg_catalog.abs(1, 2) = 1)
            AND (pg_catalog.abs(ALL 1) = 1)
            AND (pg_catalog.abs(x => 1) = 1)
            AND (pg_catalog.lower('a', 'b') = 'a')
            AND (pg_catalog.lower(ALL 'a') = 'a')
            AND (pg_catalog.lower(x => 'a') = 'a')
            AND (pg_catalog.lower(DISTINCT 'a') = 'a')
            AND (pg_catalog.substr('a') = 'a')
            AND (pg_catalog.substr(ALL 'a', 1) = 'a')
            AND (pg_catalog.substr(x => 'a', 1) = 'a')
            AND (pg_catalog.substr('a', x => 1) = 'a')
            AND (pg_catalog.substr('a', 1, x => 2) = 'a')
            AND (pg_catalog.substr('a', 1, 2, 3) = 'a')
            AND (nullif('a') = 'a')
            AND (nullif(x => 'a', 'b') = 'a')
            AND (nullif('a', x => 'b') = 'a')
            AND (pg_catalog.lower(CAST(missing.col IS NULL AS text)) = 'a')
            AND (CAST(missing.col IS NULL AS text) = 'a')
            AND (NOT (missing.col IS NULL))
            AND ((missing.col IS NULL) AND true)
            AND (true AND (missing.col IS NULL))
            AND ((missing.col IS NULL) IS DISTINCT FROM true)
            AND (true IS DISTINCT FROM (missing.col IS NULL))
            AND ((missing.col IS NULL) IS TRUE)
            AND ((missing.col IS NULL) IN (true, false));",
        "SELECT
            pg_catalog.replace(ALL 'a', 'b', 'c'),
            pg_catalog.replace('a', 'b'),
            pg_catalog.replace(1, 'b', NULL),
            pg_catalog.replace(o.name, 'b', NULL),
            pg_catalog.lag() OVER (),
            pg_catalog.lag(o.id, 0::bigint) OVER (),
            pg_catalog.sum(o.amount) OVER w2
        FROM orders o
        WINDOW w1 AS (ORDER BY o.id), w2 AS (w1);",
        "SELECT
            (SELECT a.n FROM accounts a WHERE a.id = o.id ORDER BY ALL),
            (SELECT a.n FROM accounts a WHERE a.id = o.id LIMIT ALL),
            (SELECT a.n FROM accounts a WHERE a.id = o.id LIMIT missing.n),
            (SELECT (SELECT 1) FROM accounts a WHERE a.id = o.id)
        FROM orders o;",
        "SELECT pg_catalog.count(o.id) FROM orders o WHERE false GROUP BY ROLLUP (o.id);",
        "SELECT pg_catalog.count(o.id) FROM orders o WHERE false GROUP BY GROUPING SETS (((o.id, o.account_id)), (o.id));",
        "SELECT EXISTS (SELECT (SELECT pg_catalog.count(1) FROM accounts a) FROM orders o);",
        "MERGE INTO orders o USING incoming s ON o.id = s.id
        WHEN MATCHED AND o.enabled THEN DELETE
        WHEN MATCHED AND o.enabled THEN UPDATE SET account_name = '';",
        "SELECT o.id FROM orders o GROUP BY ALL;",
        "SELECT o.id FROM orders o GROUP BY 0, 1.5;",
        "SELECT o.id FROM orders o GROUP BY GROUPING SETS (());",
        "SELECT 1 AS a, 2 AS a GROUP BY a;",
        "SELECT (SELECT a.id FROM accounts a WHERE a.id = o.id AND a.code = 01_000)
        FROM orders o
        ORDER BY (SELECT a.id FROM accounts a WHERE a.id = o.id AND a.code = 1000);",
        "SELECT pg_catalog.sum(ALL o.amount) FROM orders o ORDER BY pg_catalog.sum(o.amount);",
        "SELECT (SELECT b.id FROM accounts b WHERE b.id = a.id) FROM accounts a, accounts a;",
    ];
    let mut failures = Vec::new();
    for sql in statements {
        let facts = extract_sql_statement_facts_with_policy(
            sql,
            false,
            None,
            SqlLocationDemand::None,
            StatementPolicySources {
                collect_duplicate_keyed_lookups: true,
                ..Default::default()
            },
            false,
        );
        if facts.parse_failed {
            failures.push(sql);
        } else {
            assert!(
                facts.duplicate_keyed_lookups.is_empty(),
                "unexpected finding for: {sql}"
            );
        }
    }
    assert!(failures.is_empty(), "parse failures: {failures:#?}");
}
