use super::apply_taint;
use crate::codebase::postgres::query_annotation::evaluate::{Scope, Value};
use crate::fx::{FxHashMap, FxHashSet};

#[test]
fn shared_frames_keep_one_tainted_snapshot() {
    let shared = Scope::from(FxHashMap::from_iter([
        (
            "sql".into(),
            Value::Prefix("select 1".into(), true, Some(7)),
        ),
        (
            "other".into(),
            Value::Prefix("select 2".into(), true, Some(8)),
        ),
    ]));
    let mut scopes = vec![shared.clone(), shared.clone(), shared];
    let mut changed = FxHashSet::default();
    changed.insert(7);
    apply_taint(&mut scopes, &changed);
    assert!(scopes[0].same_bindings(&scopes[1]));
    assert!(scopes[1].same_bindings(&scopes[2]));
    assert!(matches!(scopes[0].get("sql"), Some(Value::Unknown)));
    assert!(matches!(
        scopes[2].get("other"),
        Some(Value::Prefix(_, _, Some(8)))
    ));
}
