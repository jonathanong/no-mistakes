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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_ancestor_preserves_key_and_index_suffixes() {
        let mut provenance = ConfigProvenance::default();
        let rules = ConfigPath::default().key("rules");
        let rule = rules.clone().index(3);
        let fragment = PathBuf::from(".no-mistakes/rules/types.yml");
        provenance.insert(
            rules,
            PathBuf::from(".no-mistakes.yml"),
            ConfigPath::default(),
        );
        provenance.insert(
            rule.clone(),
            fragment.clone(),
            ConfigPath::default().index(1),
        );
        let query = rule.key("options").key("roots").index(2);
        assert_eq!(
            provenance.origin(&query),
            Some((
                fragment.as_path(),
                ConfigPath::default()
                    .index(1)
                    .key("options")
                    .key("roots")
                    .index(2)
            ))
        );
        assert_eq!(
            provenance.origin(&ConfigPath::default().key("projects")),
            None
        );
    }
}
