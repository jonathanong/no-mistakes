#[test]
fn fetch_expression_counts_preserve_bound_and_ties_policy() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/fetch-expressions.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 8);
    let caps = facts
        .bounds
        .iter()
        .map(|fact| fact.query.capped)
        .collect::<Vec<_>>();
    assert_eq!(caps, [true, false, true, false, true, false, true, false]);
    let catalog = super::catalog();
    for (index, fact) in facts.bounds.iter().enumerate() {
        let names = super::offenders(fact, &catalog)
            .into_iter()
            .map(|offender| offender.table)
            .collect::<Vec<_>>();
        match index {
            0 | 2 | 4 | 6 => assert!(names.is_empty(), "statement {index}: {names:?}"),
            1 | 5 => assert_eq!(names, ["accounts"], "statement {index}"),
            3 | 7 => assert!(
                names.contains(&"accounts".to_string()),
                "statement {index}: {names:?}"
            ),
            _ => unreachable!(),
        }
    }
}
