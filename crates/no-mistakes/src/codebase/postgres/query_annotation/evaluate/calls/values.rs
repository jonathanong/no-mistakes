use super::super::{Evaluator, Value};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in super::super) fn refresh_arguments(&self, values: &mut [Value]) {
        if self.builder_updates.is_empty() && self.invalidated_builders.is_empty() {
            return;
        }
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
            Value::Aggregate(values)
            | Value::References(values)
            | Value::Object(values)
            | Value::Record(values, _)
            | Value::Joined(values)
            | Value::Possible(values) => {
                // A borrowed summary avoids detaching immutable scalar records.
                if values.contains_reference() {
                    self.refresh_arguments(values);
                }
            }
            // Argument containers already resolve their slots by identity.
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
