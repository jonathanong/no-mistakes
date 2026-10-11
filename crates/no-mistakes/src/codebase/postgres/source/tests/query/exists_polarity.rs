use super::*;

/// depth, effective, negated, mandatory, under_not, under_or, under_case,
/// under_boolean_test, under_other, correlated.
type Polarity = (
    u32,
    Option<bool>,
    bool,
    bool,
    bool,
    bool,
    bool,
    bool,
    bool,
    bool,
);

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
                .inspect(|fact| {
                    let hidden = fact.6 || fact.7 || fact.8;
                    assert_eq!(fact.1, (!hidden).then_some(fact.0 % 2 == 1));
                })
                .collect()
        })
        .collect()
}

#[test]
fn nested_not_keeps_node_negation_and_exposes_effective_polarity() {
    let report = report();
    let positive = (
        0,
        Some(false),
        false,
        true,
        false,
        false,
        false,
        false,
        false,
        false,
    );
    let keyword_not = (
        1,
        Some(true),
        true,
        false,
        true,
        false,
        false,
        false,
        false,
        false,
    );
    let wrapped_not = (
        1,
        Some(true),
        false,
        false,
        true,
        false,
        false,
        false,
        false,
        false,
    );
    assert_eq!(
        report[0],
        vec![keyword_not],
        "NOT EXISTS is logically negative"
    );
    assert_eq!(
        report[1],
        vec![(
            2,
            Some(false),
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            false,
        )],
        "NOT (NOT EXISTS) is logically positive"
    );
    assert_eq!(report[2], vec![positive]);
    assert_eq!(report[3], vec![wrapped_not], "NOT (EXISTS)");
    assert_eq!(
        report[4],
        vec![(
            2,
            Some(false),
            false,
            false,
            true,
            false,
            false,
            false,
            false,
            false,
        )],
        "NOT (NOT (EXISTS))"
    );
    assert_eq!(
        report[5],
        vec![(
            3,
            Some(true),
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            false,
        )],
        "NOT (NOT (NOT EXISTS))"
    );
    assert_eq!(
        report[6],
        vec![(
            2,
            Some(false),
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            false,
        )]
    );
    assert_eq!(
        report[7],
        vec![(
            2,
            Some(false),
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            false,
        )]
    );
    assert_eq!(report[8], vec![keyword_not, positive,]);
    assert_eq!(
        report[9],
        vec![
            (
                1,
                Some(true),
                true,
                false,
                true,
                true,
                false,
                false,
                false,
                false,
            ),
            (
                0,
                Some(false),
                false,
                false,
                false,
                true,
                false,
                false,
                false,
                false,
            ),
        ]
    );
    assert_eq!(
        report[10],
        vec![
            (
                1,
                Some(true),
                false,
                false,
                true,
                true,
                false,
                false,
                false,
                false,
            ),
            (
                1,
                Some(true),
                false,
                false,
                true,
                true,
                false,
                false,
                false,
                false,
            ),
        ]
    );
    assert_eq!(
        report[11],
        vec![
            wrapped_not,
            (
                2,
                Some(false),
                true,
                false,
                true,
                false,
                false,
                false,
                false,
                false,
            ),
        ]
    );
    assert_eq!(
        report[12],
        vec![
            (1, None, true, false, true, false, true, false, false, false),
            (0, None, false, false, false, false, true, false, false, false),
        ]
    );
    assert_eq!(
        report[13],
        vec![(1, None, false, false, true, false, true, false, false, false)]
    );
    // IS NOT TRUE is a boolean test, not another NOT operator.
    let boolean = (
        0, None, false, false, false, false, false, true, false, false,
    );
    assert_eq!(report[14], vec![boolean; 6]);
    assert_eq!(
        report[15],
        vec![
            (1, None, false, false, true, false, false, true, false, false),
            (1, None, true, false, true, false, false, true, false, false),
            (1, None, true, false, true, false, false, true, false, false),
        ]
    );
    assert_eq!(
        report[16],
        vec![
            keyword_not,
            (
                2,
                Some(false),
                true,
                false,
                true,
                false,
                false,
                false,
                false,
                false,
            ),
        ]
    );
    assert_eq!(report[17], vec![keyword_not, keyword_not]);
    assert_eq!(report[18], vec![positive]);
    assert_eq!(report[19], vec![keyword_not]);
    assert_eq!(
        report[20],
        vec![(1, None, false, false, true, false, false, false, true, false)]
    );
    assert_eq!(
        report[21],
        vec![(1, None, true, false, true, false, false, false, true, false)]
    );
    assert_eq!(
        report[22],
        vec![(
            2,
            Some(false),
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            true,
        )]
    );
    assert_eq!(
        report[23],
        vec![
            keyword_not,
            (
                2,
                Some(false),
                true,
                false,
                true,
                false,
                false,
                false,
                false,
                false,
            ),
        ]
    );
    assert_eq!(
        report[24],
        vec![
            keyword_not,
            (
                2,
                Some(false),
                true,
                false,
                true,
                false,
                false,
                false,
                false,
                false,
            ),
        ]
    );
    assert_eq!(report[25], vec![positive]);
    assert_eq!(
        report[26],
        vec![(1, None, false, false, true, false, false, false, true, false)],
        "NOT (EXISTS = false) hides polarity"
    );
    // BETWEEN, IN, and arrays hide parity the same way a comparison does.
    let wrapped = (
        1, None, false, false, true, false, false, false, true, false,
    );
    assert_eq!(
        report[27],
        vec![wrapped],
        "NOT (EXISTS BETWEEN) hides polarity"
    );
    assert_eq!(report[28], vec![wrapped], "NOT (IN list) hides polarity");
    assert_eq!(report[29], vec![wrapped], "NOT (array) hides polarity");
    assert_eq!(
        report[30],
        vec![wrapped],
        "NOT (value IN (subquery)) hides polarity"
    );

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
    assert!(nested[1].not_depth == 2 && nested[1].effective_negated == Some(false));
    let case_not = exists_in(&facts.statements[13]);
    let boolean_tests = exists_in(&facts.statements[14]);
    let compared = exists_in(&facts.statements[26]);
    for exists in [&case_not[0], &boolean_tests[2], &compared[0]] {
        let json = serde_json::to_value(exists).unwrap();
        let object = json.as_object().unwrap();
        assert!(object.contains_key("effectiveNegated"));
        assert!(json["effectiveNegated"].is_null());
        assert_eq!(json["notDepth"], exists.not_depth);
        assert_eq!(json["negated"], exists.negated);
    }
}

#[test]
fn effective_mandatory_accounts_for_de_morgan_and_opaque_wrappers() {
    let facts = facts("query-exists-effective-mandatory.sql");
    assert!(facts.diagnostics.is_empty(), "{:?}", facts.diagnostics);
    assert_eq!(facts.statements.len(), 12);
    let expected = [
        (Some(true), Some(true)),  // direct NOT EXISTS
        (Some(true), Some(false)), // NOT (false AND EXISTS)
        (Some(true), Some(true)),  // NOT (true OR EXISTS)
        (Some(false), Some(true)), // double NOT
        (Some(true), Some(true)),  // triple NOT
        (Some(true), Some(false)), // nested OR beneath disjunctive AND
        (Some(true), Some(true)),  // nested ORs beneath NOT
        (Some(true), Some(false)), // ordinary OR
        (Some(true), Some(false)), // projection, not WHERE
    ];
    for (statement, (negated, mandatory)) in facts.statements.iter().zip(expected) {
        let occurrences = exists_in(statement);
        assert_eq!(occurrences.len(), 1, "{}", statement.sql);
        assert_eq!(
            occurrences[0].effective_negated, negated,
            "{}",
            statement.sql
        );
        assert_eq!(
            occurrences[0].context.effective_mandatory, mandatory,
            "{}",
            statement.sql
        );
    }

    let branches = exists_in(&facts.statements[9]);
    assert_eq!(branches.len(), 2);
    assert_ne!(branches[0].scope_id, branches[1].scope_id);
    assert_eq!(branches[0].context.effective_mandatory, Some(true));
    assert_eq!(branches[1].context.effective_mandatory, Some(false));
    for statement in &facts.statements[10..] {
        let occurrence = &exists_in(statement)[0];
        assert_eq!(occurrence.effective_negated, None);
        assert_eq!(occurrence.context.effective_mandatory, None);
    }
    let json = serde_json::to_value(&branches[0]).unwrap();
    assert_eq!(json["context"]["effectiveMandatory"], true);
    assert_eq!(json["context"]["mandatory"], false);
}
