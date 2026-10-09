use super::{Evaluator, Value};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn index(&self, value: Value, index: usize) -> Value {
        match value {
            Value::Arguments(id, values) => {
                if self.invalidated_builders.contains(&id)
                    || self.deleted_argument_slots.contains(&(id, Some(index)))
                    || self.deleted_argument_slots.contains(&(id, None))
                {
                    Value::Unknown
                } else {
                    values.get(index).cloned().unwrap_or(Value::Unknown)
                }
            }
            value => Value::Aggregate(vec![value]),
        }
    }
}
