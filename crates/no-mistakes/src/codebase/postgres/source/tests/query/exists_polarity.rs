use super::*;

/// depth, effective, negated, mandatory, under_not, under_or, under_case,
/// under_boolean_test, under_other, correlated.
type Polarity = (u32, bool, bool, bool, bool, bool, bool, bool, bool, bool);

fn polarity(exists: &PostgresSqlQueryExists) -> Polarity {
    let context = &exists.context;
    (
        exists.not_depth,
        exists.effective_negated,
        exists.negated,
        context.mandatory,
        context.under_not,
        context.under_or,
        context.under_case,
        context.under_boolean_test,
        context.under_other,
        exists.correlated,
    )
}

fn exists_in(statement: &PostgresSqlStatement) -> Vec<PostgresSqlQueryExists> {
    let query = match &statement.facts {
        PostgresSqlStatementKind::Select { query } => query,
        PostgresSqlStatementKind::Insert { insert } => match &insert.source {
            PostgresSqlInsertSource::Select { query, .. } => query,
            other => panic!("expected INSERT SELECT, got {other:?}"),
        },
        other => panic!("expected a query statement, got {other:?}"),
    };
    assert!(query.complete, "{:?}", query.unsupported);
    let mut exists = query.exists.clone();
    exists.sort_by_key(|fact| fact.span.as_ref().map(|span| span.start.offset));
    exists
}

fn report() -> Vec<Vec<Polarity>> {
    let facts = facts("query-exists-polarity.sql");
    assert!(facts.diagnostics.is_empty(), "{:?}", facts.diagnostics);
    facts
        .statements
        .iter()
        .map(|statement| {
            exists_in(statement)
                .iter()
                .map(polarity)
                .inspect(|fact| assert_eq!(fact.1, fact.0 % 2 == 1))
                .collect()
        })
        .collect()
}

#[test]
fn nested_not_keeps_node_negation_and_exposes_effective_polarity() {
    let report = report();
    let positive = (
        0, false, false, true, false, false, false, false, false, false,
    );
    let keyword_not = (
        1, true, true, false, true, false, false, false, false, false,
    );
    let wrapped_not = (
        1, true, false, false, true, false, false, false, false, false,
    );
    assert_eq!(
        report[0],
        vec![keyword_not],
        "NOT EXISTS is logically negative"
    );
    assert_eq!(
        report[1],
        vec![(2, false, true, false, true, false, false, false, false, false)],
        "NOT (NOT EXISTS) is logically positive"
    );
    assert_eq!(report[2], vec![positive]);
    assert_eq!(report[3], vec![wrapped_not], "NOT (EXISTS)");
    assert_eq!(
        report[4],
        vec![(2, false, false, false, true, false, false, false, false, false)],
        "NOT (NOT (EXISTS))"
    );
    assert_eq!(
        report[5],
        vec![(3, true, true, false, true, false, false, false, false, false)],
        "NOT (NOT (NOT EXISTS))"
    );
    assert_eq!(
        report[6],
        vec![(2, false, true, false, true, false, false, false, false, false)]
    );
    assert_eq!(
        report[7],
        vec![(2, false, true, false, true, false, false, false, false, false)]
    );
    assert_eq!(report[8], vec![keyword_not, positive,]);
    assert_eq!(
        report[9],
        vec![
            (1, true, true, false, true, true, false, false, false, false),
            (0, false, false, false, false, true, false, false, false, false),
        ]
    );
    assert_eq!(
        report[10],
        vec![
            (1, true, false, false, true, true, false, false, false, false),
            (1, true, false, false, true, true, false, false, false, false),
        ]
    );
    assert_eq!(
        report[11],
        vec![
            wrapped_not,
            (2, false, true, false, true, false, false, false, false, false),
        ]
    );
    assert_eq!(
        report[12],
        vec![
            (1, true, true, false, true, false, true, false, false, false),
            (0, false, false, false, false, false, true, false, false, false),
        ]
    );
    assert_eq!(
        report[13],
        vec![(1, true, false, false, true, false, true, false, false, false)]
    );
    // IS NOT TRUE is a boolean test, not another NOT operator.
    let boolean = (
        0, false, false, false, false, false, false, true, false, false,
    );
    assert_eq!(report[14], vec![boolean; 6]);
    assert_eq!(
        report[15],
        vec![
            (1, true, false, false, true, false, false, true, false, false),
            (1, true, true, false, true, false, false, true, false, false),
            (1, true, true, false, true, false, false, true, false, false),
        ]
    );
    assert_eq!(
        report[16],
        vec![
            keyword_not,
            (2, false, true, false, true, false, false, false, false, false),
        ]
    );
    assert_eq!(report[17], vec![keyword_not, keyword_not]);
    assert_eq!(report[18], vec![positive]);
    assert_eq!(report[19], vec![keyword_not]);
    assert_eq!(
        report[20],
        vec![(1, true, false, false, true, false, false, false, true, false)]
    );
    assert_eq!(
        report[21],
        vec![(1, true, true, false, true, false, false, false, true, false)]
    );
    assert_eq!(
        report[22],
        vec![(2, false, true, false, true, false, false, false, false, true)]
    );
    assert_eq!(
        report[23],
        vec![
            keyword_not,
            (2, false, true, false, true, false, false, false, false, false),
        ]
    );
    assert_eq!(
        report[24],
        vec![
            keyword_not,
            (2, false, true, false, true, false, false, false, false, false),
        ]
    );
    assert_eq!(report[25], vec![positive]);

    let facts = facts("query-exists-polarity.sql");
    let exists = &exists_in(&facts.statements[0])[0];
    let json = serde_json::to_value(exists).unwrap();
    assert_eq!(exists.scope_id, 0);
    assert_eq!(json["notDepth"], 1);
    assert_eq!(json["effectiveNegated"], true);
    assert_eq!(json["negated"], true);
    assert_eq!(json["context"]["underNot"], true);
    assert_eq!(json["context"]["mandatory"], false);
    let nested = &exists_in(&facts.statements[16]);
    assert_eq!(nested[0].scope_id, 0);
    assert!(nested[1].scope_id > nested[0].scope_id);
    assert!(nested[1].not_depth == 2 && !nested[1].effective_negated);
}
