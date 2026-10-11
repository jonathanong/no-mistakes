use super::{Evaluator, Value};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn index(&self, value: Value, index: usize) -> Value {
        match value {
            Value::Arguments(id) => {
                // A later write publishes that property even if a conditional
                // deletion marker is still keeping the formal uncertain.
                if self.recreated_argument_slots.contains(&(id, index)) {
                    let slot = self
                        .argument_slot(id, index)
                        .cloned()
                        .unwrap_or(Value::Unknown);
                    return if self.invalidated_builders.contains(&id) {
                        Value::Possible(vec![slot].into())
                    } else {
                        slot
                    };
                }
                if self.definite_deleted_argument_slots.contains(&(id, index)) {
                    return Value::Unknown;
                }
                let slot = self
                    .argument_slot(id, index)
                    .cloned()
                    .unwrap_or(Value::Unknown);
                if self.invalidated_builders.contains(&id)
                    || self.deleted_argument_slots.contains(&(id, Some(index)))
                    || self.deleted_argument_slots.contains(&(id, None))
                {
                    Value::Possible(vec![slot].into())
                } else {
                    slot
                }
            }
            value => super::member::member_candidates(value),
        }
    }
}
