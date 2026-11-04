use crate::config::source_locations::{ConfigPath, Segment};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default)]
pub struct ConfigProvenance(HashMap<ConfigPath, (PathBuf, ConfigPath)>);

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
        self.0.insert(global, (path, local));
    }
    pub(crate) fn origin(&self, path: &ConfigPath) -> Option<(&Path, ConfigPath)> {
        let mut prefix = path.clone();
        let mut suffix = Vec::new();
        let (file, local) = loop {
            if let Some(origin) = self.0.get(&prefix) {
                break origin;
            }
            suffix.push(prefix.pop()?);
        };
        let mut result = local.clone();
        for segment in suffix.iter().rev() {
            result = match segment {
                Segment::Key(key) => result.key(key),
                Segment::Index(index) => result.index(*index),
            };
        }
        Some((file, result))
    }
}
