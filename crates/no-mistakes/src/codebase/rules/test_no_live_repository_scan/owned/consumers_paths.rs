//! Retain each resolved path form once for broad and test-scoped consumers.
use super::Consumed;
use crate::codebase::ts_resolver::normalize_path;
use std::path::{Path, PathBuf};

impl Consumed {
    pub(super) fn insert(&mut self, target: &Path) {
        self.insert_with_origin(None, target);
    }

    pub(super) fn insert_from(&mut self, source: &Path, target: &Path) {
        self.insert_with_origin(Some(source), target);
    }

    fn insert_with_origin(&mut self, source: Option<&Path>, target: &Path) {
        self.insert_form(source, normalize_path(target));
        if let Ok(real) = target.canonicalize() {
            self.insert_form(source, normalize_path(&real));
        }
    }

    fn insert_form(&mut self, source: Option<&Path>, target: PathBuf) {
        if let Some(source) = source {
            self.direct.push((source.to_path_buf(), target.clone()));
        }
        self.files.insert(target);
    }
}
