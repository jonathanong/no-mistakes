use crate::config::source_locations::{ConfigPath, Segment};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default)]
pub struct ConfigProvenance(Vec<(ConfigPath, PathBuf, ConfigPath)>);

impl PartialEq for ConfigProvenance {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl ConfigProvenance {
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub(crate) fn insert(&mut self, global: ConfigPath, path: PathBuf, local: ConfigPath) {
        self.0.push((global, path, local));
    }
    pub(crate) fn origin(&self, path: &ConfigPath) -> Option<(&Path, ConfigPath)> {
        let (global, file, local) = self
            .0
            .iter()
            .filter(|(global, _, _)| path.as_slice().starts_with(global.as_slice()))
            .max_by_key(|(global, _, _)| global.as_slice().len())?;
        let mut result = local.clone();
        for segment in &path.as_slice()[global.as_slice().len()..] {
            result = match segment {
                Segment::Key(key) => result.key(key),
                Segment::Index(index) => result.index(*index),
            };
        }
        Some((file, result))
    }
}
