use super::{catalog, fixture_root};
use crate::codebase::postgres::extract_sql_statement_facts;
use crate::codebase::postgres::statements::SqlBoundKind;
use crate::codebase::rules::postgres_bounded_statements::evaluate::offenders;

fn read_sql(name: &str) -> String {
    std::fs::read_to_string(fixture_root().join("sql").join(name)).unwrap()
}

fn statements(sql: &str) -> Vec<&str> {
    sql.split(';')
        .map(str::trim)
        .filter(|part| {
            part.contains("SELECT") || part.contains("UPDATE") || part.contains("DELETE")
        })
        .collect()
}

fn assert_clear(name: &str) {
    let catalog = catalog();
    let sql = read_sql(name);
    for statement in statements(&sql) {
        let facts = extract_sql_statement_facts(statement);
        assert!(!facts.parse_failed, "{name}: {statement}");
        let hits: Vec<_> = facts
            .bounds
            .iter()
            .filter(|fact| !offenders(fact, &catalog).is_empty())
            .collect();
        assert!(hits.is_empty(), "{name}: {statement}");
    }
}

#[test]
fn issue_2961_accepted_shapes_stay_bounded() {
    for name in [
        "issue-2961-bounded-source.sql",
        "issue-2961-union-all.sql",
        "issue-2961-any-array.sql",
        "issue-2961-unqualified-column.sql",
        "issue-2961-delete-array.sql",
    ] {
        assert_clear(name);
    }
}

#[test]
fn issue_2961_counterexamples_stay_unbounded() {
    let expected = [
        SqlBoundKind::Update,
        SqlBoundKind::Update,
        SqlBoundKind::Update,
        SqlBoundKind::Update,
        SqlBoundKind::Select,
        SqlBoundKind::Update,
    ];
    let catalog = catalog();
    let sql = read_sql("issue-2961-counterexamples.sql");
    let parts = statements(&sql);
    assert_eq!(parts.len(), expected.len());
    for (statement, kind) in parts.iter().zip(expected) {
        let facts = extract_sql_statement_facts(statement);
        assert!(!facts.parse_failed, "{statement}");
        let hits: Vec<_> = facts
            .bounds
            .iter()
            .filter(|fact| !offenders(fact, &catalog).is_empty())
            .collect();
        assert_eq!(hits.len(), 1, "{statement}");
        assert_eq!(hits[0].kind, kind, "{statement}");
    }
}
