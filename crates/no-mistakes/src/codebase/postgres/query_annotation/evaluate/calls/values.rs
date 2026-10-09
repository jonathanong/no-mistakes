use super::super::{Evaluator, Value};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn refresh_arguments(&self, values: &mut [Value]) {
        for value in values {
            self.refresh_argument(value);
        }
    }

    fn refresh_argument(&self, value: &mut Value) {
        match value {
            Value::Prefix(_, _, Some(id)) => {
                if self.invalidated_builders.contains(id) {
                    *value = Value::Unknown;
                } else if let Some(current) = self.builder_updates.get(id) {
                    *value = current.clone();
                }
            }
            Value::Promise(inner) | Value::Evaluated(inner, _) => {
                self.refresh_argument(inner);
            }
            Value::Aggregate(values) => self.refresh_arguments(values),
            // Argument containers already resolve their slots by identity.
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
