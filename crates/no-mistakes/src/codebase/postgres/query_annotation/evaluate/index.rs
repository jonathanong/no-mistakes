use super::{Evaluator, Value};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn index(&self, value: Value, index: usize) -> Value {
        match value {
            Value::Arguments(id) => {
                if self.definite_deleted_argument_slots.contains(&(id, index)) {
                    return Value::Unknown;
                }
                let slot = self
                    .argument_objects
                    .get(&id)
                    .and_then(|values| values.get(index))
                    .cloned()
                    .unwrap_or(Value::Unknown);
                if self.invalidated_builders.contains(&id)
                    || self.deleted_argument_slots.contains(&(id, Some(index)))
                    || self.deleted_argument_slots.contains(&(id, None))
                {
                    Value::Aggregate(vec![Value::Unknown, slot])
                } else {
                    slot
                }
            }
            value => Value::Aggregate(vec![value]),
        }
    }
}
