use crate::codebase::postgres::query_annotation::evaluate::Scope;
use crate::codebase::postgres::query_annotation::evaluate::Value;
use crate::fx::FxHashMap;

const UNPROVEN: Value = Value::Unknown;

/// A binding absent from one arm is not SQL proof.
pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) fn binding_or_unproven<
    'a,
>(
    scope: &'a FxHashMap<String, Value>,
    name: &str,
) -> &'a Value {
    scope.get(name).unwrap_or(&UNPROVEN)
}

/// Preserves an annotated builder only when the original and both arms agree.
pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) fn join_with_sql(
    joined: &mut [Scope],
    current: &[Scope],
    original: &[Scope],
    whole_sql: bool,
) {
    for ((before, after), initial) in joined.iter_mut().zip(current).zip(original) {
        let changed = before
            .iter()
            .filter(|(name, value)| {
                after.get(*name) != Some(*value) || !initial.contains_key(*name)
            })
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>();
        for name in changed {
            let value = before.get_mut(&name).expect("existing binding");
            *value =
                super::super::join_value(initial.get(&name), value, after.get(&name), whole_sql);
        }
    }
}
