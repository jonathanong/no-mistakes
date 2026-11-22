use super::super::{Evaluator, Value};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in super::super) fn refresh_arguments(&self, values: &mut [Value]) {
        if self.invalidated_builders.is_empty() && self.builder_updates.is_empty() {
            return;
        }
        for value in values {
            self.refresh_argument(value);
        }
    }

    fn refresh_argument(&self, value: &mut Value) {
        #[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
        super::super::effects::call_value_work_tests::record_refresh_visit();
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
            | Value::Possible(values)
                if values.contains_reference() =>
            {
                for value in values {
                    self.refresh_argument(value);
                }
            }
            // Argument containers already resolve their slots by identity.
            _ => {}
        }
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;
