use crate::codebase::structured_value::parse_structured_value;
use serde_yaml::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Default)]
pub(in crate::codebase::rules::structured_config_policy) struct ParsedAncestorCache {
    pub(in crate::codebase::rules::structured_config_policy) values:
        BTreeMap<PathBuf, Result<Arc<Value>, String>>,
}

impl ParsedAncestorCache {
    pub(super) fn parse(&mut self, path: &Path, source: &str) -> Result<Arc<Value>, String> {
        if let Some(value) = self.values.get(path) {
            return value.clone();
        }
        let value = parse_structured_value(path, source)
            .map(Arc::new)
            .map_err(|error| error.to_string());
        self.values.insert(path.to_path_buf(), value.clone());
        value
    }
}

#[cfg(test)]
mod tests;
