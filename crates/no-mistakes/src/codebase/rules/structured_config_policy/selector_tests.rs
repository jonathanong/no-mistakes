use super::*;

#[test]
fn match_any_groups_root_arrays_and_indexed_rest() {
    let root_array: serde_yaml::Value =
        serde_yaml::from_str("- name: keep\n- name: drop\n").unwrap();
    let root_any = assert_value(
        "app.yml",
        &root_array,
        &ValueAssertion {
            key: "[].name".to_string(),
            kind: Some(AssertionKind::Equals),
            match_mode: MatchMode::Any,
            value: Some(serde_yaml::Value::String("keep".to_string())),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(root_any.is_empty(), "{root_any:?}");

    let items: serde_yaml::Value = serde_yaml::from_str(
        r#"
items:
  - name: keep
  - extra: 1
"#,
    )
    .unwrap();
    let rest = assert_value(
        "app.yml",
        &items,
        &ValueAssertion {
            key: "items.[].name".to_string(),
            kind: Some(AssertionKind::Equals),
            match_mode: MatchMode::Any,
            value: Some(serde_yaml::Value::String("keep".to_string())),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(rest.is_empty(), "{rest:?}");

    let missing = assert_value(
        "app.yml",
        &items,
        &ValueAssertion {
            key: "missing.[].name".to_string(),
            kind: Some(AssertionKind::Equals),
            match_mode: MatchMode::Any,
            value: Some(serde_yaml::Value::String("keep".to_string())),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(missing.is_empty(), "{missing:?}");

    let empty_array: serde_yaml::Value =
        serde_yaml::from_str("overrides:\n  - rules:\n      no-restricted-properties: []\n")
            .unwrap();
    let empty = assert_value(
        "app.yml",
        &empty_array,
        &ValueAssertion {
            key: "overrides.[].rules.no-restricted-properties.[]".to_string(),
            kind: Some(AssertionKind::ObjectShape),
            match_mode: MatchMode::Any,
            required_values: [(
                "property".to_string(),
                serde_yaml::Value::String("bind".to_string()),
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(empty.len(), 1, "{empty:?}");

    let indexed = assert_value(
        "app.yml",
        &items,
        &ValueAssertion {
            key: "items.9.name".to_string(),
            kind: Some(AssertionKind::Equals),
            match_mode: MatchMode::Any,
            value: Some(serde_yaml::Value::String("keep".to_string())),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(indexed.is_empty(), "{indexed:?}");

    let not_array = assert_value(
        "app.yml",
        &serde_yaml::from_str("name: keep\n").unwrap(),
        &ValueAssertion {
            key: "name.[]".to_string(),
            kind: Some(AssertionKind::Equals),
            match_mode: MatchMode::Any,
            value: Some(serde_yaml::Value::String("keep".to_string())),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(!not_array.is_empty(), "{not_array:?}");

    let not_seq_index = assert_value(
        "app.yml",
        &serde_yaml::from_str("name: keep\n").unwrap(),
        &ValueAssertion {
            key: "name.0".to_string(),
            kind: Some(AssertionKind::Equals),
            match_mode: MatchMode::Any,
            value: Some(serde_yaml::Value::String("keep".to_string())),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(not_seq_index.is_empty(), "{not_seq_index:?}");
}

fn any_equals(key: &str, root: &serde_yaml::Value) -> Vec<crate::codebase::rules::RuleFinding> {
    assert_value(
        "app.yml",
        root,
        &ValueAssertion {
            key: key.to_string(),
            kind: Some(AssertionKind::Equals),
            match_mode: MatchMode::Any,
            value: Some(serde_yaml::Value::String("keep".to_string())),
            ..Default::default()
        },
    )
    .unwrap()
}

#[test]
fn match_any_skips_non_array_wildcards_inside_parent_walk() {
    let items: serde_yaml::Value = serde_yaml::from_str(
        r#"
items:
  - name: keep
  - extra: 1
"#,
    )
    .unwrap();
    assert!(any_equals("items.[].tags.[]", &items).is_empty());
    assert_eq!(any_equals("items.[].0", &items).len(), 1);
}
