use super::{facts, fixture};
use serde_json::{json, Value};

fn projected(name: &str) -> Value {
    serde_json::to_value(facts(name)).unwrap()
}

#[test]
fn trigger_events_preserve_update_column_identity_and_unrestricted_update() {
    let result = projected("replay-structured-triggers.sql");
    assert_eq!(result["diagnostics"], json!([]));
    let trigger = &result["statements"][0]["trigger"];
    assert_eq!(trigger["events"], json!(["UPDATE OF \"Mixed\", a"]));
    assert_eq!(trigger["eventFacts"][0]["kind"], "update");
    assert_eq!(trigger["eventFacts"][0]["updateOf"], true);
    assert_eq!(
        trigger["eventFacts"][0]["updateColumns"],
        json!([
            {"value":"Mixed","quoted":true,"identity":"Mixed"},
            {"value":"a","quoted":false,"identity":"a"}
        ])
    );
    assert_eq!(
        result["statements"][1]["trigger"]["eventFacts"],
        json!([
            {"kind":"update","updateOf":false,"updateColumns":[]}
        ])
    );
    assert_eq!(
        result["statements"][2]["trigger"]["eventFacts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|event| event["kind"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["insert", "delete", "truncate"]
    );
}

#[test]
fn literal_execute_concatenation_and_using_share_the_nested_typed_pipeline() {
    let sql = fixture("replay-structured-execute.sql");
    let result = projected("replay-structured-execute.sql");
    assert_eq!(result["diagnostics"], json!([]));
    let block = &result["statements"][0]["block"];
    assert_eq!(block["complete"], true);
    for statement in block["statements"].as_array().unwrap() {
        assert_eq!(statement["kind"], "literalExecute");
        let execute = &statement["execute"];
        assert_eq!(execute["complete"], true);
        assert_eq!(execute["diagnostics"], json!([]));
        assert_eq!(execute["statements"][0]["kind"], "insert");
        assert_eq!(
            execute["statements"][0]["insert"]["onConflict"]["action"]["kind"],
            "doNothing"
        );
        let span = &execute["literalSpan"];
        let source = &sql[span["start"]["offset"].as_u64().unwrap() as usize
            ..span["end"]["offset"].as_u64().unwrap() as usize];
        assert!(source.starts_with("'INSERT INTO t "));
        let nested = &execute["statements"][0];
        let decoded = execute["decodedSql"].as_str().unwrap();
        assert_eq!(
            &decoded[nested["span"]["start"]["offset"].as_u64().unwrap() as usize
                ..nested["span"]["end"]["offset"].as_u64().unwrap() as usize],
            nested["sql"]
        );
    }
    let statements = &block["statements"];
    assert_eq!(statements[0]["execute"]["bodyEncoding"], "concatenated");
    assert_eq!(statements[0]["execute"]["using"], json!([]));
    assert_eq!(
        statements[1]["execute"]["using"][0]["root"]["value"],
        json!({"kind":"number","value":"1"})
    );
    assert_eq!(
        statements[1]["execute"]["statements"][0]["insert"]["source"]["rows"][0][0]["root"],
        json!({"kind":"parameter","placeholder":"$1"})
    );
    let unsupported = projected("replay-structured-execute-unsupported.sql");
    assert_eq!(unsupported["statements"][0]["block"]["complete"], false);
    assert!(unsupported["statements"][0]["block"]["statements"]
        .as_array()
        .unwrap()
        .iter()
        .all(|statement| statement["kind"] == "other"));
}

#[test]
fn casts_and_boolean_children_are_structurally_complete_without_claiming_spans() {
    let result = projected("replay-structured-casts.sql");
    assert_eq!(result["diagnostics"], json!([]));
    let assignments = &result["statements"][0]["insert"]["onConflict"]["action"]["assignments"];
    for index in 0..6 {
        assert_eq!(
            assignments[index]["expression"]["childrenComplete"], true,
            "{}",
            assignments[index]
        );
    }
    let root = &assignments[0]["expression"]["root"];
    assert_eq!(root["dataType"], "\"date\"");
    assert_eq!(
        root["dataTypeFacts"]["name"]["parts"],
        json!([{"value":"date","quoted":true,"identity":"date"}])
    );
    let qualified = &assignments[1]["expression"]["root"]["dataTypeFacts"]["name"]["parts"];
    assert_eq!(
        qualified,
        &json!([
            {"value":"schema.with.dot","quoted":true,"identity":"schema.with.dot"},
            {"value":"date","quoted":false,"identity":"date"}
        ])
    );
    assert_eq!(
        assignments[2]["expression"]["children"][0]["children"][0]["root"]["dataTypeFacts"]["name"]
            ["parts"],
        *qualified
    );
    for index in [3, 4] {
        let expression = &assignments[index]["expression"];
        assert_eq!(expression["children"][0]["root"]["kind"], "nullTest");
        assert_eq!(expression["children"][0]["span"], Value::Null);
        assert_eq!(expression["children"][1]["span"], Value::Null);
        // Structural completeness is not sufficient proof for legacy assignment provenance.
        assert_eq!(assignments[index]["complete"], false);
    }
    assert_eq!(assignments[6]["expression"]["childrenComplete"], false);
}

#[test]
fn literal_execute_variants_retain_parameters_and_recover_malformed_neighbors() {
    let sql = fixture("replay-structured-execute-variants.sql");
    let result = projected("replay-structured-execute-variants.sql");
    let block = &result["statements"][0]["block"];
    assert_eq!(block["complete"], false);
    let statements = block["statements"].as_array().unwrap();
    assert_eq!(statements.len(), 9, "{result}");
    let execute = &statements[0]["execute"];
    assert_eq!(execute["complete"], true);
    assert_eq!(
        execute["decodedSql"],
        "INSERT INTO t\nVALUES($1) ON CONFLICT DO NOTHING"
    );
    assert_eq!(
        execute["using"][0]["root"],
        json!({"kind":"parameter","placeholder":"$2"})
    );
    assert_eq!(
        execute["using"][1]["root"]["dataTypeFacts"]["builtin"],
        "numeric"
    );
    let span = &execute["literalSpan"];
    assert_eq!(
        &sql[span["start"]["offset"].as_u64().unwrap() as usize
            ..span["end"]["offset"].as_u64().unwrap() as usize],
        "(E'INSERT INTO t\\n' || ($sql$VALUES($1) $sql$ || 'ON CONFLICT DO NOTHING'))"
    );
    assert_eq!(statements[1]["execute"]["complete"], false);
    assert_eq!(statements[1]["execute"]["statements"][0]["kind"], "insert");
    for index in [2, 3, 4, 5, 7] {
        assert_eq!(statements[index]["kind"], "other");
    }
    for (index, decoded) in [
        (6, "INSERT INTO t VALUES(2)"),
        (8, "INSERT INTO t VALUES(4)"),
    ] {
        assert_eq!(statements[index]["kind"], "literalExecute");
        assert_eq!(statements[index]["execute"]["decodedSql"], decoded);
    }
}
