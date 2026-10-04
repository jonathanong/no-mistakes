use super::names;
use super::offenders;
use crate::codebase::postgres::{extract_sql_statement_facts, SchemaCatalog};

#[test]
fn temporary_partitions_only_expose_permanent_names_after_cascade() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-bounded-statements")
            .join("sql/temporary-partitions.sql"),
    )
    .unwrap();
    assert!(crate::codebase::postgres::extract_sql_statement_facts(&sql).parse_failed);
    assert_eq!(names(&sql), ["orders", "orders"]);
}

#[test]
fn partition_ddl_uses_catalog_evidence_for_the_parent_and_child() {
    let root = crate::test_support::rule_fixture_root("postgres-bounded-statements");
    let catalog = SchemaCatalog::from_json(
        &std::fs::read_to_string(root.join("schema-partition-search-path-evidence.json")).unwrap(),
    )
    .unwrap();
    for (fixture, expected) in [
        (
            "temporary-partition-conditional-empty.sql",
            vec![vec![], vec!["orders"], vec!["orders"]],
        ),
        (
            "temporary-partition-conditional-physical.sql",
            vec![vec!["orders"], vec![]],
        ),
        (
            "temporary-partition-conditional-qualified.sql",
            vec![vec!["orders"], vec![], vec!["orders"]],
        ),
    ] {
        let sql = std::fs::read_to_string(root.join("sql").join(fixture)).unwrap();
        let facts = extract_sql_statement_facts(&sql);
        let projected = crate::codebase::postgres::project_sql_bounds(&facts, &catalog);
        assert_eq!(projected.len(), expected.len(), "{fixture}");
        let actual = projected
            .iter()
            .map(|fact| {
                offenders(fact, &catalog)
                    .into_iter()
                    .map(|offender| offender.table)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "{fixture}");
    }

    // This cross-persistence ATTACH is rejected by PostgreSQL. Its temporary
    // child still selects per-catalog replay, but the physical parent owns nothing.
    let sql =
        std::fs::read_to_string(root.join("sql/temporary-partition-conditional-rejected.sql"))
            .unwrap();
    let facts = extract_sql_statement_facts(&sql);
    assert!(facts.lifecycle.is_some());
    let projected = crate::codebase::postgres::project_sql_bounds(&facts, &catalog);
    assert_eq!(projected.len(), 1);
    assert!(offenders(&projected[0], &catalog).is_empty());
}
