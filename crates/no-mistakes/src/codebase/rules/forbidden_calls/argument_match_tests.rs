use super::*;
use serde_json::json;

fn condition(value: serde_json::Value) -> ArgumentConditions {
    serde_json::from_value(json!({"all": [value]})).unwrap()
}
fn scalar(value: serde_json::Value) -> ArgumentValue {
    ArgumentValue::Scalar(value)
}
fn check(predicate: serde_json::Value, value: ArgumentValue) -> Match {
    matches(
        &condition(predicate),
        Some(&ArgumentValues {
            values: vec![value],
            spread_from: None,
        }),
    )
}

#[test]
fn scalar_literal_types_and_known_incompatible_shapes_are_distinct() {
    for value in [json!("check"), json!(1), json!(true), json!(null)] {
        assert_eq!(
            check(json!({"position": 0, "equals": value}), scalar(value)),
            Match::Yes
        );
    }
    assert_eq!(
        check(json!({"position":0,"equals":1.0}), scalar(json!(1))),
        Match::Yes
    );
    assert_eq!(
        check(json!({"position":0,"equals":"1"}), scalar(json!(1))),
        Match::No
    );
    assert_eq!(
        check(
            json!({"position":0,"equals":null}),
            ArgumentValue::Array(vec![])
        ),
        Match::No
    );
    assert_eq!(
        check(json!({"position":0,"equals":null}), ArgumentValue::Unknown),
        Match::Unknown
    );
}

#[test]
fn array_exclusion_requires_complete_negative_evidence() {
    let values = ArgumentValue::Array(vec![scalar(json!("check")), ArgumentValue::Unknown]);
    assert_eq!(
        check(
            json!({"position":0,"arrayIncludes":"check"}),
            values.clone()
        ),
        Match::Yes
    );
    assert_eq!(
        check(
            json!({"position":0,"arrayExcludes":"check"}),
            values.clone()
        ),
        Match::No
    );
    assert_eq!(
        check(
            json!({"position":0,"arrayIncludes":"--root"}),
            values.clone()
        ),
        Match::Unknown
    );
    assert_eq!(
        check(json!({"position":0,"arrayExcludes":"--root"}), values),
        Match::Unknown
    );
    assert_eq!(
        check(
            json!({"position":0,"arrayExcludes":"--root"}),
            ArgumentValue::Array(vec![])
        ),
        Match::Yes
    );
    assert_eq!(
        check(
            json!({"position":0,"arrayExcludes":"--root"}),
            scalar(json!(1))
        ),
        Match::No
    );
    assert_eq!(
        check(
            json!({"position":0,"arrayIncludes":"--root"}),
            ArgumentValue::Unknown
        ),
        Match::Unknown
    );
}

#[test]
fn property_missing_position_spread_and_conjunction_keep_three_values() {
    let predicate = json!({"position":0,"property":"cwd","equals":"/workspace"});
    let object = ArgumentValue::Object(
        [(String::from("cwd"), scalar(json!("/workspace")))]
            .into_iter()
            .collect(),
    );
    assert_eq!(check(predicate.clone(), object), Match::Yes);
    assert_eq!(
        check(predicate.clone(), ArgumentValue::Object(Default::default())),
        Match::No
    );
    assert_eq!(check(predicate.clone(), scalar(json!(1))), Match::No);
    assert_eq!(
        check(predicate.clone(), ArgumentValue::Unknown),
        Match::Unknown
    );
    let condition = condition(predicate);
    assert_eq!(matches(&condition, None), Match::Unknown);
    assert_eq!(
        matches(
            &condition,
            Some(&ArgumentValues {
                values: vec![],
                spread_from: None
            })
        ),
        Match::No
    );
    assert_eq!(
        matches(
            &condition,
            Some(&ArgumentValues {
                values: vec![],
                spread_from: Some(0)
            })
        ),
        Match::Unknown
    );
    let conditions: ArgumentConditions = serde_json::from_value(
        json!({"all":[{"position":0,"equals":1},{"position":1,"equals":2}]}),
    )
    .unwrap();
    assert_eq!(
        matches(
            &conditions,
            Some(&ArgumentValues {
                values: vec![ArgumentValue::Unknown, scalar(json!(3))],
                spread_from: None
            })
        ),
        Match::No
    );
}

#[test]
fn configured_predicate_limits_fail_closed() {
    for conditions in [
        json!([]),
        json!([{"position":32,"equals":1}]),
        json!([{"position":0,"property":"","equals":1}]),
        json!([{"position":0,"property":"x".repeat(129),"equals":1}]),
        json!([{"position":0,"equals":"x".repeat(4097)}]),
        serde_json::Value::Array(vec![json!({"position":0,"equals":1}); 17]),
    ] {
        let options: super::super::config::Options = serde_json::from_value(json!({"roots":[{"file":"repro.mjs"}],"targets":[{"exact":"execute","whenArguments":{"all":conditions}}]})).unwrap();
        assert!(super::super::config::validate(&options).is_err());
    }
    assert!(serde_json::from_value::<ArgumentConditions>(
        json!({"all":[{"position":0,"equals":1,"arrayIncludes":1}]})
    )
    .is_err());
}
