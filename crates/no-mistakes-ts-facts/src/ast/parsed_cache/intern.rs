use super::ParsedProgramCache;
use std::path::{Path, PathBuf};
use std::sync::Arc;

impl ParsedProgramCache {
    pub(super) fn intern_path(&self, path: PathBuf) -> Arc<Path> {
        {
            let interned = self.interned_paths.borrow();
            if let Some((existing, _)) = interned.get_key_value(path.as_path()) {
                return Arc::clone(existing);
            }
        }
        let interned = Arc::<Path>::from(path);
        self.interned_paths
            .borrow_mut()
            .insert(Arc::clone(&interned), ());
        interned
    }

    pub(super) fn interned_lookup(&self, path: &Path) -> Option<Arc<Path>> {
        self.interned_paths
            .borrow()
            .get_key_value(path)
            .map(|(existing, _)| Arc::clone(existing))
    }
}
