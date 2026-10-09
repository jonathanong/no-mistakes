use super::{Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::Expr;
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn deleted(
        &mut self,
        children: &[Expr],
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
            self.invalidated_builders.insert(*id);
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
