use crate::codebase::postgres::{parse_postgres_sql, SchemaCatalog};
use sqlparser::ast::Statement;
use std::collections::BTreeMap;

#[test]
fn threshold_absence_rewrites_require_physical_correlated_key_only_queries() {
    let root = crate::test_support::rule_fixture_root(super::RULE_ID).join("existence-boundaries");
    let catalog =
        SchemaCatalog::from_json(&std::fs::read_to_string(root.join("schema.json")).unwrap())
            .unwrap();
    let expected: BTreeMap<String, usize> =
        serde_json::from_str(&std::fs::read_to_string(root.join("expected.json")).unwrap())
            .unwrap();
    let actual = expected
        .keys()
        .map(|name| {
            let sql = std::fs::read_to_string(root.join(format!("{name}.sql"))).unwrap();
            let statements =
                parse_postgres_sql(&sql).unwrap_or_else(|error| panic!("{name}: {error}"));
            let [Statement::Query(query)] = statements.as_slice() else {
                panic!("{name}: expected one query")
            };
            let findings: Vec<_> = super::analyze::query(query, &catalog, &[])
                .into_iter()
                .filter(|finding| {
                    finding.code == "aggregate-existence-test"
                        && finding.message.starts_with("MAX ")
                })
                .collect();
            for finding in &findings {
                assert!(
                    finding.message.contains("preserve the original NOT EXISTS"),
                    "{name}: {}",
                    finding.message
                );
                assert!(
                    finding.message.contains("threshold IS NOT NULL"),
                    "{name}: {}",
                    finding.message
                );
            }
            (name.clone(), findings.len())
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(actual, expected);
}
