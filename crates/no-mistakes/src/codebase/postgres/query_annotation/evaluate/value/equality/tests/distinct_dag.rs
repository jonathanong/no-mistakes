use super::{hooks, Value};
use crate::fx::FxHashMap;
use serde::Deserialize;

#[derive(Deserialize)]
struct Node {
    name: String,
    sql: Option<String>,
    #[serde(default)]
    children: Vec<String>,
}
#[derive(Deserialize)]
struct Fixture {
    nodes: Vec<Node>,
    left: String,
    right: String,
}
fn pair() -> (Value, Value) {
    let fixture: Fixture = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/performance/annotation-shared-record-dag/value-equality-distinct/state.json"
    )))
    .unwrap();
    let mut values = FxHashMap::<String, Value>::default();
    for node in fixture.nodes {
        let value = if let Some(sql) = node.sql {
            Value::Prefix(sql, true, None)
        } else {
            Value::Record(
                node.children
                    .iter()
                    .map(|name| values[name].clone())
                    .collect::<Vec<_>>()
                    .into(),
                vec!["a".into(), "b".into()],
            )
        };
        values.insert(node.name, value);
    }
    (
        values.remove(&fixture.left).unwrap(),
        values.remove(&fixture.right).unwrap(),
    )
}

#[test]
fn equal_distinct_saved_dag_compares_each_container_pair_once() {
    let (left, right) = pair();
    let (equal, legacy_work) = hooks::measure(true, || left == right);
    assert!(equal);
    let (equal, work) = hooks::measure(false, || left == right);
    assert!(equal);
    eprintln!("distinct DAG fallback comparisons: {legacy_work} -> {work}");
    assert!(legacy_work > 8_000);
    assert!(work <= 30, "shared descendants must not expand into a tree");
}

fn assert_oracle(left: &Value, right: &Value, expected: bool) {
    assert_eq!(left == right, expected);
    assert_eq!(hooks::measure(true, || left == right).0, expected);
    assert_eq!(super::legacy::equal(left, right), expected);
}

#[test]
fn paired_dags_keep_variant_and_wrapper_contracts() {
    let (left, right) = pair();
    let wrappers: [fn(Value) -> Value; 7] = [
        |value| Value::Promise(Box::new(value)),
        |value| Value::Evaluated(Box::new(value), true),
        |value| Value::Aggregate(vec![value].into()),
        |value| Value::References(vec![value].into()),
        |value| Value::Object(vec![value].into()),
        |value| Value::Joined(vec![value].into()),
        |value| Value::Possible(vec![value].into()),
    ];
    for wrap in wrappers {
        assert_oracle(&wrap(left.clone()), &wrap(right.clone()), true);
    }
    assert_oracle(
        &Value::Aggregate(vec![left].into()),
        &Value::Object(vec![right].into()),
        false,
    );
}

#[test]
fn completed_content_pairs_do_not_prove_record_labels() {
    let (Value::Record(left, _), Value::Record(right, _)) = pair() else {
        panic!("saved DAG roots must be records");
    };
    let wrap = |values, label: &str| Value::Record(values, vec![label.into()]);
    let left = Value::Aggregate(vec![wrap(left.clone(), "first"), wrap(left, "second")].into());
    let right =
        Value::Aggregate(vec![wrap(right.clone(), "first"), wrap(right, "different")].into());
    assert_oracle(&left, &right, false);
}

fn replace_last_leaf(value: &mut Value) {
    match value {
        Value::Record(values, _) => replace_last_leaf(values.last_mut().unwrap()),
        Value::Prefix(sql, _, _) => sql.push_str(" WHERE false"),
        _ => panic!("saved DAG contains only records and SQL leaves"),
    }
}

#[test]
fn memo_is_local_and_copy_on_write_changes_remain_visible() {
    let (left, mut right) = pair();
    assert_oracle(&left, &right, true);
    let original = right.clone();
    replace_last_leaf(&mut right);
    assert_oracle(&left, &right, false);
    assert_oracle(&left, &original, true);
    let Value::Record(values, _) = &mut right else {
        unreachable!()
    };
    values.pop();
    assert_oracle(&left, &right, false);
}

#[test]
fn scalar_prefix_is_not_restarted_and_unique_parents_share_one_context() {
    let (left, right) = pair();
    let wrap = |child| Value::Record(vec![child].into(), vec!["child".into()]);
    let outer = |dag: Value| {
        let mut values = vec![Value::Number(7); 64];
        // Each parent owns a different container; only their descendants share.
        values.extend([wrap(dag.clone()), wrap(dag)]);
        Value::Aggregate(values.into())
    };
    let (left, right) = (outer(left), outer(right));
    let (equal, old_work) = hooks::measure(true, || left == right);
    assert!(equal);
    let (equal, work) = hooks::measure(false, || left == right);
    assert!(equal);
    eprintln!("prefix and unique-parent comparisons: {old_work} -> {work}");
    assert!(old_work > 16_000);
    assert!(
        work <= 96,
        "never restart the scalar prefix or discard the memo between siblings"
    );
    assert_oracle(&left, &right, true);
}

#[test]
fn unequal_scalar_prefix_stops_before_the_complex_suffix() {
    let (left, right) = pair();
    let left = Value::Aggregate(vec![Value::Number(1), left].into());
    let right = Value::Aggregate(vec![Value::Number(2), right].into());
    assert_eq!(hooks::measure(false, || left == right), (false, 2));
    assert_oracle(&left, &right, false);
}
