use super::Evaluator;
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};

pub(super) fn collect<'a>(names: impl Iterator<Item = &'a String>) -> FxHashMap<String, usize> {
    names
        .enumerate()
        .filter(|(_, name)| !name.is_empty())
        .map(|(index, name)| (name.clone(), index))
        .collect()
}
impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn mapped_parameter_index(
        &self,
        id: u64,
        params: &[String],
        name: &str,
    ) -> Option<usize> {
        let index = *self.mapped_parameter_indices.get(&id)?.get(name)?;
        (params.get(index).is_some_and(|param| param == name)).then_some(index)
    }
}

#[cfg(test)]
mod tests;
