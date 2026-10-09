use super::{Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::{DeleteKey, Expr};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn deleted(
        &mut self,
        children: &[Expr],
        key: DeleteKey,
        path: &Path,
        env: &Environment,
        context: (u8, bool),
    ) -> Value {
        let (depth, generic) = context;
        let values = children
            .iter()
            .map(|child| self.expr(child, path, env, depth, generic))
            .collect::<Vec<_>>();
        if let Some(Value::Arguments(id, _)) = values.first() {
            // Removing a slot does not mutate the builder it referenced.
            match key {
                DeleteKey::Index(index) => {
                    self.deleted_argument_slots.insert((*id, Some(index)));
                }
                DeleteKey::Named => {}
                DeleteKey::Dynamic => {
                    self.deleted_argument_slots.insert((*id, None));
                }
            }
            self.invalidate_builders(&values[1..]);
            self.opaque_callbacks(&values[1..], depth);
            // The complete receiver/key effects are handled precisely here.
            return Value::SlotDeletion;
        } else {
            self.invalidate_builders(&values);
            self.opaque_callbacks(&values, depth);
        }
        Value::Unknown
    }
}

impl Value {
    pub(super) fn handled_deletion(&self) -> bool {
        match self {
            Self::SlotDeletion => true,
            Self::Aggregate(values) => {
                !values.is_empty() && values.iter().all(Self::handled_deletion)
            }
            _ => false,
        }
    }
}

#[cfg(test)]
#[path = "delete/tests.rs"]
mod tests;
