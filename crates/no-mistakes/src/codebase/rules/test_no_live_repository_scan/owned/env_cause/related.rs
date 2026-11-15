use super::Owners;
use crate::codebase::dependencies::extract::is_indexable;
use crate::codebase::dependencies::graph::{EdgeKind, NodeId};
use crate::fx::FxHashSet;
use std::path::{Path, PathBuf};

#[cfg(test)]
thread_local! {
    static RELATED_FILE_COLLECTIONS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(super) fn take_related_file_collections() -> u32 {
    RELATED_FILE_COLLECTIONS.with(|count| count.replace(0))
}

impl Owners<'_> {
    pub(super) fn related_files(&self, file: &Path) -> Vec<PathBuf> {
        if let Some(cached) = self.related_files_cache.borrow().get(file) {
            return cached.clone();
        }
        let files = self.collect_related_files(file);
        self.related_files_cache
            .borrow_mut()
            .insert(file.to_path_buf(), files.clone());
        files
    }

    fn collect_related_files(&self, file: &Path) -> Vec<PathBuf> {
        #[cfg(test)]
        RELATED_FILE_COLLECTIONS.with(|count| count.set(count.get() + 1));
        let mut pending = vec![NodeId::file(file)];
        pending.extend(self.setups.setup_files_for(file).iter().map(NodeId::file));
        let mut seen = FxHashSet::default();
        let mut files = Vec::new();
        while let Some(current) = pending.pop() {
            if !seen.insert(current.clone()) {
                continue;
            }
            let NodeId::File(path) = &current else {
                continue;
            };
            let script = is_indexable(path.as_ref());
            if path.as_ref() != file && script {
                files.push(path.as_ref().to_path_buf());
            }
            if !script {
                continue;
            }
            for (node, kind) in self
                .graph
                .dependencies_of_node(&current)
                .into_iter()
                .flatten()
            {
                if *kind == EdgeKind::Import {
                    pending.push(node.clone());
                }
            }
        }
        files.sort();
        files.dedup();
        files
    }
}
