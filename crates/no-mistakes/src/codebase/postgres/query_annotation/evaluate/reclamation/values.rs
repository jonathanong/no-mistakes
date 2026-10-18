use super::super::Value;
use super::*;

impl References {
    pub(super) fn value(value: &Value) -> Self {
        fn collect(value: &Value, found: &mut References, seen: &mut FxHashSet<*const ()>) {
            match value {
                Value::Arguments(id) | Value::Array(id) => {
                    found.arguments.insert(*id);
                }
                Value::Prefix(_, _, Some(id)) => {
                    found.builders.insert(*id);
                }
                Value::Promise(value) | Value::Evaluated(value, _) => collect(value, found, seen),
                Value::Aggregate(values)
                | Value::References(values)
                | Value::Object(values)
                | Value::Record(values, _)
                | Value::Joined(values)
                | Value::Possible(values)
                    if values.contains_reference() && seen.insert(values.identity()) =>
                {
                    for value in values {
                        collect(value, found, seen);
                    }
                }
                _ => {}
            }
        }
        let mut result = Self::default();
        collect(value, &mut result, &mut FxHashSet::default());
        result
    }
    pub(super) fn identities(&self) -> impl Iterator<Item = u64> + '_ {
        self.arguments.iter().chain(&self.builders).copied()
    }
}
