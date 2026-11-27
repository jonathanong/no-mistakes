use super::{facts, fixture};
use serde_json::{json, Value};

fn projected(name: &str) -> Value {
    serde_json::to_value(facts(name)).unwrap()
}

fn slice<'a>(sql: &'a str, span: &Value) -> &'a str {
    let start = span["start"]["offset"].as_u64().unwrap() as usize;
    let end = span["end"]["offset"].as_u64().unwrap() as usize;
    &sql[start..end]
}

fn roles(expression: &Value) -> Vec<Value> {
    expression["children"]
        .as_array()
        .unwrap()
        .iter()
        .map(|child| json!([child["role"], child["index"]]))
        .collect()
}

#[test]
fn insert_recursive_roles_order_and_legacy_summaries_are_distinct() {
    let sql = fixture("insert-recursive-expressions.sql");
    let result = projected("insert-recursive-expressions.sql");
    assert_eq!(result["diagnostics"], json!([]));
    let assignments = result["statements"][0]["insert"]["onConflict"]["action"]["assignments"]
        .as_array()
        .unwrap();
    assert_eq!(assignments.len(), 13);
    let forward = &assignments[0]["expression"];
    let reversed = &assignments[1]["expression"];
    assert_eq!(
        roles(forward),
        vec![json!(["argument", 0]), json!(["argument", 1])]
    );
    assert_eq!(forward["children"][0]["root"]["kind"], "columnReference");
    assert_eq!(reversed["children"][0]["root"]["kind"], "functionCall");
    assert_eq!(forward["childrenComplete"], true);
    assert_eq!(assignments[2]["provenance"], "excludedColumn");
    assert_eq!(assignments[2]["expression"]["children"], json!([]));
    let binary = &assignments[3]["expression"];
    assert_eq!(
        roles(binary),
        vec![json!(["binaryLeft", null]), json!(["binaryRight", null])]
    );
    assert_eq!(binary["childrenComplete"], true);
    // Recursive completeness does not change the legacy syntax-only contract.
    assert_eq!(assignments[3]["complete"], false);
    assert_eq!(
        binary["children"][0]["children"][0]["children"][0]["root"]["name"]["sql"],
        "EXCLUDED.value"
    );
    assert_eq!(
        slice(&sql, &binary["children"][0]["span"]),
        "lower(COALESCE(EXCLUDED.value, target.value))"
    );
    assert_eq!(slice(&sql, &binary["children"][1]["span"]), "upper('λ')");
    assert_eq!(
        roles(&assignments[4]["expression"]),
        vec![
            json!(["caseOperand", null]),
            json!(["caseWhenCondition", 0]),
            json!(["caseWhenResult", 0]),
            json!(["caseWhenCondition", 1]),
            json!(["caseWhenResult", 1]),
            json!(["caseElse", null]),
        ]
    );
    assert_eq!(
        roles(&assignments[5]["expression"]),
        vec![
            json!(["caseWhenCondition", 0]),
            json!(["caseWhenResult", 0])
        ]
    );
    for index in [7, 8, 9, 11] {
        assert_eq!(assignments[index]["expression"]["childrenComplete"], false);
    }
    assert_eq!(
        assignments[10]["expression"]["children"][0]["argumentName"]["identity"],
        "first_arg"
    );
    assert_eq!(assignments[12]["provenance"], "targetColumn");
    let wrapped = &assignments[6];
    assert_eq!(wrapped["complete"], true);
    assert_eq!(
        slice(&sql, &wrapped["expression"]["span"]),
        "-(CAST((COALESCE(target.id, EXCLUDED.id)) AS integer))"
    );
    let operand = &wrapped["expression"]["children"][0];
    if operand["span"].is_null() {
        assert_eq!(wrapped["expression"]["childrenComplete"], true);
    } else {
        assert_eq!(
            slice(&sql, &operand["span"]),
            "(CAST((COALESCE(target.id, EXCLUDED.id)) AS integer))"
        );
    }
}

#[test]
fn insert_column_sources_keep_rows_branches_and_proven_bytes() {
    let sql = fixture("insert-column-sources.sql");
    let result = projected("insert-column-sources.sql");
    assert_eq!(result["diagnostics"], json!([]));
    let statements = result["statements"].as_array().unwrap();
    assert_eq!(statements.len(), 7);
    for statement in &statements[..3] {
        let mapping = &statement["insert"]["columnSources"];
        assert_eq!(mapping["kind"], "mapped");
        assert_eq!(mapping["complete"], true);
        assert_eq!(mapping["columns"][0]["columnIndex"], 0);
        assert_eq!(mapping["columns"][1]["column"]["sql"], "value");
    }
    let sources = &statements[0]["insert"]["columnSources"]["columns"][1]["sources"];
    assert_eq!(sources[0]["rowIndex"], 0);
    assert_eq!(sources[1]["rowIndex"], 1);
    assert_eq!(sources[0]["branchPath"], json!([]));
    assert_eq!(
        sources[0]["expression"],
        statements[0]["insert"]["source"]["rows"][0][1]
    );
    assert_eq!(
        slice(&sql, &sources[0]["expression"]["span"]),
        "COALESCE('λ', lower('first'))"
    );
    let branches = statements[1]["insert"]["columnSources"]["columns"][1]["sources"]
        .as_array()
        .unwrap();
    assert_eq!(
        branches
            .iter()
            .map(|source| source["branchPath"].clone())
            .collect::<Vec<_>>(),
        vec![json!([0]), json!([1, 0]), json!([1, 1])]
    );
    assert_eq!(
        slice(&sql, &branches[2]["expression"]["span"]),
        "upper('right')"
    );
    let mixed = &statements[2]["insert"]["columnSources"]["columns"][1]["sources"];
    assert_eq!(mixed[0]["kind"], "values");
    assert_eq!(mixed[1]["kind"], "select");
    for (index, expected) in [
        (3, "-(CAST((COALESCE(1, 2)) AS integer))"),
        (4, "DATE '2026-10-08'"),
        (5, "sum(1) OVER ()"),
        (6, "-(1) + 2"),
    ] {
        let mapping = &statements[index]["insert"]["columnSources"];
        let expression = &mapping["columns"][1]["sources"][0]["expression"];
        if expression["span"].is_null() {
            assert_eq!(expression["childrenComplete"], index != 5);
            assert_eq!(mapping["complete"], false);
        } else {
            assert_eq!(slice(&sql, &expression["span"]), expected);
        }
    }
    // The parser span starts at the operand. The leading minus and parenthesis
    // are still part of the rendered source bytes.
    let signed =
        &statements[6]["insert"]["columnSources"]["columns"][1]["sources"][0]["expression"];
    assert_eq!(slice(&sql, &signed["span"]), "-(1) + 2");
}

#[test]
fn insert_column_sources_report_precise_unsupported_boundaries() {
    let result = projected("insert-column-sources-unsupported.sql");
    assert_eq!(result["diagnostics"], json!([]));
    let mappings = result["statements"]
        .as_array()
        .unwrap()
        .iter()
        .map(|statement| &statement["insert"]["columnSources"])
        .collect::<Vec<_>>();
    let reasons = [
        "columnsOmitted",
        "columnsOmitted",
        "wildcardProjection",
        "wildcardProjection",
        "sourceArityMismatch",
        "sourceArityMismatch",
        "duplicateTargetColumn",
        "sourceArityMismatch",
    ];
    for (mapping, reason) in mappings[..8].iter().zip(reasons) {
        assert_eq!(mapping["kind"], "unsupported");
        assert_eq!(mapping["reason"], reason);
    }
    assert_eq!(mappings[4]["rowIndex"], 1);
    assert_eq!(mappings[4]["expectedColumns"], 2);
    assert_eq!(mappings[4]["sourceColumns"], 1);
    assert_eq!(mappings[7]["branchPath"], json!([1]));
    assert_eq!(mappings[8]["kind"], "mapped");
    assert_eq!(mappings[8]["complete"], false);
    assert_eq!(mappings[9]["reason"], "setOperationByName");
    assert_eq!(mappings[9]["branchPath"], json!([]));
    assert_eq!(mappings[10]["branchPath"], json!([1]));
    assert_eq!(mappings[10]["rowIndex"], 0);
}

#[test]
fn insert_recursive_wrapper_and_decoded_execute_spans_use_owning_sources() {
    let sql = fixture("insert-recursive-nested.sql");
    let result = projected("insert-recursive-nested.sql");
    assert_eq!(result["diagnostics"], json!([]));
    let wrapped = &result["statements"][0]["wrapper"]["statements"][0]["insert"];
    let execute = &result["statements"][1]["block"]["statements"][0]["execute"];
    for (owner, insert) in [
        (&sql[..], wrapped),
        (
            execute["decodedSql"].as_str().unwrap(),
            &execute["statements"][0]["insert"],
        ),
    ] {
        let expression = &insert["columnSources"]["columns"][1]["sources"][0]["expression"];
        assert_eq!(expression["childrenComplete"], true);
        assert_eq!(
            slice(owner, &expression["span"]),
            "COALESCE(lower('λ'), CURRENT_TIMESTAMP)"
        );
        assert_eq!(
            slice(owner, &expression["children"][0]["span"]),
            "lower('λ')"
        );
        assert_eq!(
            slice(owner, &expression["children"][0]["children"][0]["span"]),
            "'λ'"
        );
    }
}

#[test]
fn insert_recursive_malformed_expression_preserves_neighboring_statement() {
    let result = projected("insert-recursive-invalid.sql");
    assert_eq!(result["diagnostics"].as_array().unwrap().len(), 1);
    assert_eq!(result["statements"].as_array().unwrap().len(), 1);
    assert_eq!(result["statements"][0]["kind"], "select");
    assert_eq!(result["statements"][0]["sql"], "SELECT 42 AS recovered;");
}

#[test]
fn insert_recursive_zero_argument_call_trivia_is_not_a_partial_span() {
    let sql = fixture("insert-recursive-call-trivia.sql");
    let result = projected("insert-recursive-call-trivia.sql");
    assert_eq!(result["diagnostics"], json!([]));
    let expression =
        &result["statements"][0]["insert"]["onConflict"]["action"]["assignments"][0]["expression"];
    assert_eq!(
        slice(&sql, &expression["span"]),
        "COALESCE(items.value, now /*keep*/ ())"
    );
    let partial =
        &result["statements"][2]["insert"]["onConflict"]["action"]["assignments"][0]["expression"];
    assert_eq!(
        slice(&sql, &partial["children"][1]["span"]),
        "EXCLUDED.value"
    );
    let expr_named =
        &result["statements"][3]["insert"]["onConflict"]["action"]["assignments"][0]["expression"];
    assert_eq!(expr_named["childrenComplete"], false);
    assert_eq!(expr_named["children"][0]["childrenComplete"], false);
    assert_eq!(
        expr_named["children"][0]["root"]["argumentsComplete"],
        false
    );
    assert_eq!(
        slice(&sql, &expr_named["children"][1]["span"]),
        "target.value"
    );
    let child = &expression["children"][1];
    if child["span"].is_null() {
        assert_eq!(expression["childrenComplete"], false);
    } else {
        assert_eq!(slice(&sql, &child["span"]), "now /*keep*/ ()");
    }
    let mapping = &result["statements"][1]["insert"]["columnSources"];
    let source = &mapping["columns"][1]["sources"][0]["expression"];
    if source["span"].is_null() {
        assert_eq!(source["childrenComplete"], false);
        assert_eq!(mapping["complete"], false);
    } else {
        assert_eq!(slice(&sql, &source["span"]), "now /*keep*/ ()");
    }
}
