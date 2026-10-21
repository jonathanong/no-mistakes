use super::{Environment, Value};
use crate::fx::FxHashSet;

/// Direct immutable references only: argument contents remain mutable arena state.
#[derive(Default)]
pub(super) struct Edges {
    pub environments: FxHashSet<Environment>,
    pub arguments: FxHashSet<u64>,
    pub identities: FxHashSet<u64>,
}

impl Edges {
    pub(super) fn collect<'a>(values: impl IntoIterator<Item = &'a Value>) -> Self {
        let mut edges = Self::default();
        let mut visited = FxHashSet::default();
        for value in values {
            edges.value(value, &mut visited);
        }
        edges
    }

    fn value(&mut self, value: &Value, visited: &mut FxHashSet<*const ()>) {
        match value {
            Value::Function(_, _, env) => {
                self.environments.insert(*env);
            }
            Value::Arguments(id) | Value::Array(id) => {
                self.arguments.insert(*id);
            }
            Value::Prefix(_, _, Some(id)) => {
                self.identities.insert(*id);
            }
            Value::Promise(value) | Value::Evaluated(value, _) => self.value(value, visited),
            Value::Aggregate(values)
            | Value::References(values)
            | Value::Object(values)
            | Value::Record(values, _)
            | Value::Joined(values)
            | Value::Possible(values)
                if (values.contains_reference() || values.contains_environment())
                    && visited.insert(values.identity()) =>
            {
                for value in values {
                    self.value(value, visited);
                }
            }
            _ => {}
        }
    }
}
