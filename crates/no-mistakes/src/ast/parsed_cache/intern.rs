use super::{ParseMode, ParsedProgramCache, Program};
use std::path::{Path, PathBuf};
use std::sync::Arc;

impl ParsedProgramCache {
    pub(crate) fn with_existing_program<T>(
        &self,
        path: &Path,
        analyze: impl for<'a> FnOnce(&'a Program<'a>, &'a str) -> T,
    ) -> Option<T> {
        let path = crate::codebase::ts_resolver::normalize_path(path);
        let interned = self.interned_lookup(&path)?;
        let cached = self
            .entries
            .borrow()
            .get(&(interned, ParseMode::Standard))?
            .as_ref()
            .ok()?
            .clone();
        cached.with_dependent(|owner, parsed| {
            parsed
                .strict_error
                .is_none()
                .then(|| analyze(&parsed.program, &owner.source))
        })
    }

    pub(crate) fn clear(&self) {
        self.entries.borrow_mut().clear();
        self.interned_paths.borrow_mut().clear();
    }

    pub(crate) fn remove_path(&self, path: &Path) {
        let path = crate::codebase::ts_resolver::normalize_path(path);
        if let Some(path) = self.interned_lookup(&path) {
            let mut entries = self.entries.borrow_mut();
            entries.remove(&(Arc::clone(&path), ParseMode::Standard));
            entries.remove(&(Arc::clone(&path), ParseMode::TypeScriptFallback));
            entries.remove(&(path, ParseMode::LegacySymbols));
        }
    }

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
