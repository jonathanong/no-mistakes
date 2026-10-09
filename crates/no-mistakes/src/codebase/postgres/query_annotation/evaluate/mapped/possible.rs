//! A possibly removed property cannot prove that its formal received a write.
use super::{Evaluator, Value};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(in crate::codebase::postgres::query_annotation::evaluate) fn write_possibly_disconnected_mapped_slot(
        &mut self,
        id: u64,
        index: usize,
        stored: &Value,
    ) {
        let Some((owner, name)) = self.mapped_slot_target(id, index) else {
            return;
        };
        for frame in self.captured_write_targets(owner, &name) {
            let previous = self.scopes[frame][&name].clone();
            // Keep the actual property precise; only the uncertain formal joins.
            self.scopes[frame].insert(
                name.clone(),
                Value::Possible(vec![previous, stored.clone()]),
            );
        }
    }
}
