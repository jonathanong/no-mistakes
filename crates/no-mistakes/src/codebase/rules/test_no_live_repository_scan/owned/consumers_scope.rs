//! Project already-resolved consumer evidence onto the test execution files.
use super::Consumed;
use crate::fx::{FxHashMap, FxHashSet};
use std::path::PathBuf;

impl Consumed {
    pub(in super::super) fn narrowed(&self, files: &FxHashSet<PathBuf>) -> Self {
        let mut selected = Self {
            direct: self
                .direct
                .iter()
                .filter(|(file, _)| files.contains(file))
                .cloned()
                .collect(),
            hits: self
                .hits
                .iter()
                .filter(|hit| files.contains(&hit.file))
                .cloned()
                .collect(),
            unknown_files: self.unknown_files.intersection(files).cloned().collect(),
            unknown_sites: self
                .unknown_sites
                .iter()
                .filter(|(file, _)| files.contains(file))
                .cloned()
                .collect(),
            via: self.via.clone(),
            ..Self::default()
        };
        selected
            .files
            .extend(selected.direct.iter().map(|(_, target)| target.clone()));
        let mut children: FxHashMap<_, Vec<_>> = FxHashMap::default();
        for (child, barrel) in &self.via {
            children.entry(barrel).or_default().push(child);
        }
        let mut pending: Vec<_> = selected.files.iter().cloned().collect();
        while let Some(barrel) = pending.pop() {
            for child in children.get(&barrel).into_iter().flatten() {
                if selected.files.insert((*child).clone()) {
                    pending.push((*child).clone());
                }
            }
        }
        selected
    }
}
