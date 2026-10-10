use super::{Evaluator, Value};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn member(&self, object: Value, name: &str) -> Value {
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
            // Preserve conservative member effects for other object shapes.
            value => Value::Aggregate(vec![value].into()),
        }
    }
}
