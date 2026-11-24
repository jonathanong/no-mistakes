use super::remap_scope;
use crate::codebase::postgres::query_annotation::evaluate::{value::Values, Scope, Value};
use crate::codebase::postgres::query_annotation::Function;
use crate::fx::FxHashMap;
use std::path::PathBuf;
use std::sync::Arc;

fn dummy_fn(env: usize) -> Value {
    let function = Arc::new(Function {
        start: 0,
        params: Vec::new(),
        body: Vec::new(),
        supported: true,
        asynchronous: false,
        arrow: true,
        self_name: None,
        append_callback: None,
    });
    Value::Function(function, Arc::new(PathBuf::from("a.mts")), env)
}

#[test]
fn remap_scope_remaps_all_container_variants_and_preserves_scalars() {
    let f = dummy_fn(0);
    let mut values = FxHashMap::default();
    values.insert("fn".into(), f.clone());
    values.insert("promise".into(), Value::Promise(Box::new(f.clone())));
    values.insert(
        "evaluated".into(),
        Value::Evaluated(Box::new(f.clone()), true),
    );
    values.insert(
        "aggregate".into(),
        Value::Aggregate(Values::from(vec![f.clone()])),
    );
    values.insert(
        "references".into(),
        Value::References(Values::from(vec![f.clone()])),
    );
    values.insert(
        "object".into(),
        Value::Object(Values::from(vec![f.clone()])),
    );
    values.insert(
        "record".into(),
        Value::Record(Values::from(vec![f.clone()]), vec!["k".into()]),
    );
    values.insert(
        "joined".into(),
        Value::Joined(Values::from(vec![f.clone()])),
    );
    values.insert(
        "possible".into(),
        Value::Possible(Values::from(vec![f.clone()])),
    );
    values.insert("scalar".into(), Value::Number(42));
    let scope = Scope::from(values);

    let unchanged = remap_scope(&scope, &[0]);
    assert!(unchanged.same_bindings(&scope));

    let remapped = remap_scope(&scope, &[5]);
    assert!(!remapped.same_bindings(&scope));
    let expected = dummy_fn(5);
    assert!(remapped["fn"] == expected);
    assert!(remapped["promise"] == Value::Promise(Box::new(expected.clone())));
    assert!(remapped["evaluated"] == Value::Evaluated(Box::new(expected.clone()), true));
    assert!(remapped["aggregate"] == Value::Aggregate(Values::from(vec![expected.clone()])));
    assert!(remapped["references"] == Value::References(Values::from(vec![expected.clone()])));
    assert!(remapped["object"] == Value::Object(Values::from(vec![expected.clone()])));
    assert!(
        remapped["record"] == Value::Record(Values::from(vec![expected.clone()]), vec!["k".into()])
    );
    assert!(remapped["joined"] == Value::Joined(Values::from(vec![expected.clone()])));
    assert!(remapped["possible"] == Value::Possible(Values::from(vec![expected])));
    assert!(remapped["scalar"] == Value::Number(42));
}
