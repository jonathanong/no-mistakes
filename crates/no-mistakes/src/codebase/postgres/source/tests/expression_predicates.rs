use super::*;

#[test]
fn expression_facts_retain_predicate_parameter_and_typed_literal_structure() {
    let facts = facts("expression-predicates-and-temporal-values.sql");
    assert!(facts.diagnostics.is_empty(), "{:?}", facts.diagnostics);
    let value = serde_json::to_value(facts).unwrap();
    let assignments = value["statements"][0]["insert"]["onConflict"]["action"]["assignments"]
        .as_array()
        .unwrap();
    let assignment = |name: &str| {
        assignments
            .iter()
            .find(|assignment| assignment["columns"][0]["parts"][0]["identity"] == name)
            .unwrap_or_else(|| panic!("missing assignment {name}"))
    };
    let expression = |name: &str| &assignment(name)["expression"];

    assert_eq!(expression("null_check")["root"]["kind"], "nullTest");
    assert_eq!(expression("null_check")["root"]["negated"], false);
    assert_eq!(expression("not_null_check")["root"]["negated"], true);
    assert_eq!(expression("distinct_check")["root"]["kind"], "distinctness");
    assert_eq!(expression("distinct_check")["root"]["negated"], false);
    assert_eq!(expression("not_distinct_check")["root"]["negated"], true);
    assert_eq!(
        expression("distinct_check")["children"]
            .as_array()
            .unwrap()
            .iter()
            .map(|child| child["role"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["distinctLeft", "distinctRight"]
    );

    let and = expression("and_check");
    assert_eq!(and["root"]["kind"], "binary");
    assert_eq!(and["children"][0]["root"]["kind"], "nullTest");
    assert_eq!(and["children"][1]["root"]["kind"], "nullTest");
    assert_eq!(and["children"][0]["span"], serde_json::Value::Null);
    assert_eq!(and["children"][1]["span"], serde_json::Value::Null);
    assert_eq!(and["childrenComplete"], true);
    assert_ne!(
        and["children"][0]["children"][0]["span"],
        serde_json::Value::Null
    );
    let or = expression("or_check");
    assert_eq!(or["root"]["kind"], "parenthesized");
    assert_eq!(or["children"][0]["children"][0]["root"]["kind"], "nullTest");
    let case = expression("case_check");
    assert_eq!(case["root"]["kind"], "case");
    assert_eq!(
        case["children"][1]["children"][0]["root"]["kind"],
        "parameter"
    );
    assert_eq!(
        case["children"][1]["children"][0]["root"]["placeholder"],
        "$2"
    );
    assert_eq!(
        case["children"][2]["children"][0]["root"]["placeholder"],
        "$1"
    );

    let parameters = expression("parameter_value")["children"][0]["children"]
        .as_array()
        .unwrap();
    assert_eq!(parameters[0]["children"][0]["root"]["placeholder"], "$3");
    assert_eq!(parameters[1]["root"]["placeholder"], "$4");

    for (name, sql, value) in [
        (
            "null_literal",
            "NULL",
            serde_json::json!({ "kind": "null" }),
        ),
        (
            "string_null_literal",
            "'NULL'",
            serde_json::json!({ "kind": "string", "value": "NULL" }),
        ),
        (
            "boolean_literal",
            "true",
            serde_json::json!({ "kind": "boolean", "value": true }),
        ),
        (
            "number_literal",
            "900719925474099312345",
            serde_json::json!({ "kind": "number", "value": "900719925474099312345" }),
        ),
        (
            "escaped_string_literal",
            "'it''s'",
            serde_json::json!({ "kind": "string", "value": "it's" }),
        ),
    ] {
        let root = &expression(name)["root"];
        assert_eq!(root["kind"], "literal", "{name}");
        assert_eq!(root["sql"], sql, "{name}");
        assert_eq!(root["value"], value, "{name}");
    }
    let nested_literals = expression("nested_literals")["children"]
        .as_array()
        .unwrap();
    assert_eq!(nested_literals[0]["root"]["value"]["kind"], "null");
    assert_eq!(nested_literals[1]["root"]["value"]["value"], "NULL");
    assert_eq!(nested_literals[2]["root"]["value"]["value"], true);
    assert_eq!(
        nested_literals[3]["root"]["value"]["value"],
        "900719925474099312345"
    );
    assert_eq!(nested_literals[4]["root"]["value"]["value"], "it's");
    assert_eq!(nested_literals[4]["sql"], "'it''s'");
    let case_literals = expression("case_literals")["children"].as_array().unwrap();
    assert_eq!(case_literals[0]["root"]["value"]["value"], true);
    assert_eq!(case_literals[1]["root"]["value"]["kind"], "null");
    assert_eq!(case_literals[2]["root"]["value"]["value"], "NULL");
    let literal_variants = expression("literal_variants")["children"]
        .as_array()
        .unwrap();
    assert_eq!(literal_variants[0]["root"]["value"]["value"], "it's");
    assert_eq!(
        literal_variants[1]["root"]["value"]["value"],
        "dollar 'quoted'"
    );
    assert_eq!(literal_variants[2]["root"]["value"]["kind"], "other");
    assert_eq!(literal_variants[2]["root"]["value"]["sql"], "X'AB'");
    assert_eq!(literal_variants[3]["root"]["value"]["value"], "snowman");
    assert_eq!(literal_variants[4]["root"]["value"]["value"], "café");

    for (name, data_type, literal) in [
        ("timestamp_value", "TIMESTAMP", "now"),
        ("timestamp_epoch_value", "TIMESTAMP", "epoch"),
        ("date_value", "DATE", "today"),
        (
            "timestamp_local_value",
            "TIMESTAMP WITHOUT TIME ZONE",
            "2025-01-02 03:04:05",
        ),
        (
            "timestamp_tz_value",
            "TIMESTAMP WITH TIME ZONE",
            "2025-01-02 03:04:05+00",
        ),
        ("time_local_value", "TIME WITHOUT TIME ZONE", "03:04:05"),
        ("time_tz_value", "TIME WITH TIME ZONE", "03:04:05+00"),
    ] {
        let root = &expression(name)["root"];
        assert_eq!(root["kind"], "typedLiteral", "{name}");
        assert_eq!(root["dataType"], data_type, "{name}");
        assert_eq!(root["value"], literal, "{name}");
        assert_eq!(root["sql"], format!("{data_type} '{literal}'"), "{name}");
    }
}
