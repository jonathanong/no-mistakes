use super::*;
use std::sync::Arc;

fn named(scope: Option<usize>, offset: u32, context: Option<CallableId>) -> ValueExpression {
    ValueExpression::Named {
        name: "client".into(),
        scope,
        offset,
        context,
    }
}

#[test]
fn exact_candidate_hashing_covers_values_and_map_order_without_erasing_identity() {
    let base = Arc::new(named(Some(0), 4, None));
    let mut fields = fx_map();
    fields.insert("a".into(), (*base).clone());
    fields.insert("b".into(), ValueExpression::Null);
    let mut reversed = fx_map();
    reversed.insert("b".into(), ValueExpression::Null);
    reversed.insert("a".into(), (*base).clone());
    for (left, right) in [
        (
            ValueExpression::Array(Arc::new(fields.clone())),
            ValueExpression::Array(Arc::new(reversed.clone())),
        ),
        (
            ValueExpression::Object(Arc::new(fields), true),
            ValueExpression::Object(Arc::new(reversed), true),
        ),
    ] {
        assert_eq!(left, right);
        assert_eq!(fingerprint(&(0, left)), fingerprint(&(0, right)));
    }
    let values = vec![
        ValueExpression::Derived(vec![ValueExpression::Null, ValueExpression::Undefined].into()),
        ValueExpression::Derived(vec![ValueExpression::Undefined, ValueExpression::Null].into()),
        (*base).clone(),
        named(Some(1), 4, None),
        named(Some(0), 5, None),
        named(Some(0), 4, Some(CallableId(1))),
        named(None, 4, None),
        ValueExpression::Call(1, 2),
        ValueExpression::Callable(CallableId(1)),
        ValueExpression::Super(CallableId(1)),
        ValueExpression::Awaited(Arc::clone(&base)),
        ValueExpression::Member(Arc::clone(&base), "get".into()),
        ValueExpression::Nullish(Arc::clone(&base), Arc::new(ValueExpression::Null)),
        ValueExpression::Default(Arc::clone(&base), Arc::new(ValueExpression::Undefined)),
        ValueExpression::Either(base, Arc::new(ValueExpression::Unknown)),
        ValueExpression::Undefined,
        ValueExpression::Null,
        ValueExpression::Unknown,
    ];
    let count = values.len();
    let mut candidates = super::super::candidates::Candidates::default();
    for value in values {
        candidates.insert((0, value.clone()));
        candidates.insert((0, value));
    }
    assert_eq!(candidates.items.len(), count);
    assert!(!candidates.contains(&(99, ValueExpression::Unknown)));
}
