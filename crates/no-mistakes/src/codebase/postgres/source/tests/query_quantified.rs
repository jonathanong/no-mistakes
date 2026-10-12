use super::super::PostgresSqlStatementKind;
use super::*;

#[test]
fn scalar_quantified_comparisons_are_typed_and_complete() {
    let sql = fixture("query-scalar-quantified.sql");
    let value = serde_json::to_value(facts("query-scalar-quantified.sql")).unwrap();
    assert_eq!(value["diagnostics"], serde_json::json!([]));
    let statements = value["statements"].as_array().unwrap();
    assert_eq!(statements.len(), 9);
    assert_eq!(statements[0]["insert"]["complete"], true);
    assert_eq!(statements[8]["insert"]["complete"], true);
    for (index, (operator, quantifier)) in [
        ("=", "any"),
        ("<>", "all"),
        ("<", "any"),
        ("<=", "all"),
        (">", "any"),
        (">=", "all"),
        ("=", "any"),
    ]
    .iter()
    .enumerate()
    {
        let query = if index == 0 {
            &statements[index]["insert"]["source"]["query"]
        } else {
            &statements[index]["query"]
        };
        assert_eq!(query["complete"], true, "{query}");
        let comparison = &query["quantifiedComparisons"][0];
        assert_eq!(comparison["operator"], *operator);
        assert_eq!(comparison["quantifier"], *quantifier);
        assert_eq!(comparison["complete"], true);
        assert_eq!(comparison["clause"], "where");
        assert_eq!(comparison["scopeId"], 0);
        for operand in ["left", "right"] {
            assert_eq!(comparison[operand]["childrenComplete"], true);
        }
        let span = &comparison["span"];
        assert!(!span.is_null(), "{comparison}");
        {
            let start = span["start"]["offset"].as_u64().unwrap() as usize;
            let end = span["end"]["offset"].as_u64().unwrap() as usize;
            assert!(sql[start..end].contains(if *quantifier == "any" { "ANY" } else { "ALL" }));
        }
    }
    let array = &statements[2]["query"]["quantifiedComparisons"][0]["right"];
    assert_eq!(array["root"]["kind"], "array");
    assert_eq!(array["children"][0]["role"], "arrayElement");
    assert_eq!(array["children"][1]["index"], 1);
    let contexts = &statements[7]["query"]["quantifiedComparisons"];
    assert_eq!(contexts[0]["joinId"], 0);
    assert_eq!(contexts[1]["context"]["underNot"], true);
    assert_eq!(contexts[2]["context"]["underOr"], true);
    // Quantified equality is never an ordinary column equality.
    assert_eq!(statements[7]["query"]["equalities"], serde_json::json!([]));
}

#[test]
fn unsupported_quantified_operands_and_malformed_syntax_fail_closed() {
    let facts = facts("query-scalar-quantified-unsupported.sql");
    assert!(facts.diagnostics.is_empty(), "{:?}", facts.diagnostics);
    assert_eq!(facts.statements.len(), 13);
    for statement in facts.statements {
        let PostgresSqlStatementKind::Select { query } = statement.facts else {
            panic!()
        };
        assert!(!query.complete);
        assert!(!query.unsupported.is_empty());
        assert!(query
            .quantified_comparisons
            .iter()
            .all(|fact| !fact.complete));
    }
    assert!(!super::facts("query-scalar-quantified-malformed.sql")
        .diagnostics
        .is_empty());
}
