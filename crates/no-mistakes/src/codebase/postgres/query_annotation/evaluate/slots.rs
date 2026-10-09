use super::{Evaluator, Value};
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn argument_slots(&self, id: u64) -> impl Iterator<Item = (usize, &Value)> {
        self.argument_objects[&id].iter().enumerate().chain(
            self.argument_extra_slots
                .get(&id)
                .into_iter()
                .flat_map(|slots| slots.iter().map(|(index, value)| (*index, value))),
        )
    }

    pub(super) fn argument_slot(&self, id: u64, index: usize) -> Option<&Value> {
        self.argument_objects[&id].get(index).or_else(|| {
            self.argument_extra_slots
                .get(&id)
                .and_then(|slots| slots.get(&index))
        })
    }
}
