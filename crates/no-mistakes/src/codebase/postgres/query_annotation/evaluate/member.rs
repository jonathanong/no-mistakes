use super::{Evaluator, Value};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn member(&self, object: Value, name: &str, path: &Path) -> Value {
        match object {
            // The intrinsic length is a primitive; passing it cannot expose
            // an argument slot or a callback captured by the container.
            Value::Arguments(id) if name == "length" => {
                if self.invalidated_builders.contains(&id) {
                    Value::Unknown
                } else {
                    Value::Primitive
                }
            }
            Value::Record(values, keys) if self.files[path].facts.precise_records => keys
                .iter()
                .rposition(|key| key == name)
                .and_then(|index| values.get(index).cloned())
                .unwrap_or(Value::Unknown),
            // Preserve conservative member effects for other object shapes.
            value => member_candidates(value),
        }
    }
}

pub(super) fn member_candidates(value: Value) -> Value {
    match value.exposed() {
        Value::Record(values, _) => Value::Aggregate(values),
        Value::Object(values) => Value::Aggregate(values),
        Value::Aggregate(values) | Value::Joined(values) | Value::Possible(values) => {
            Value::Aggregate(
                values
                    .into_iter()
                    .map(member_candidates)
                    .collect::<Vec<_>>()
                    .into(),
            )
        }
        value => Value::Aggregate(vec![value].into()),
    }
}
