use super::{finding, relative_slash_path, Walk};
use crate::codebase::structured_value::{parse_structured_value_located, StructuredValueError};
use serde_yaml::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Default)]
pub(in crate::codebase::rules::structured_config_policy) struct ParsedAncestorCache {
    pub(in crate::codebase::rules::structured_config_policy) values:
        BTreeMap<PathBuf, Result<Arc<Value>, StructuredValueError>>,
}

impl ParsedAncestorCache {
    pub(super) fn parse(&mut self, path: &Path, source: &str) -> Result<Arc<Value>, StructuredValueError> {
        if let Some(value) = self.values.get(path) {
            return value.clone();
        }
        let value = parse_structured_value_located(path, source).map(Arc::new);
        self.values.insert(path.to_path_buf(), value.clone());
        value
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;

impl Walk<'_> {
    pub(super) fn load(&mut self, spec: &str, resolved: &Path) -> Option<Arc<Value>> {
        let Some(source) = crate::codebase::rules::read_language_source(self.sources, resolved) else {
            self.findings.push(finding(
                self.nested_rel,
                self.assertion,
                format!(
                    "{}: ancestor-override-subset `{spec}` is missing",
                    self.nested_rel
                ),
            ));
            return None;
        };
        match self.parsed_ancestors.parse(resolved, &source) {
            Ok(value) => Some(value),
            Err(mut error) if error.missing_merge_location => {
                let available = error.partial.take();
                self.findings.push(
                    crate::codebase::rules::structured_config_policy::merge_location_finding(
                        self.root, resolved, error,
                    ),
                );
                if let Some(available) = available {
                    let relative = relative_slash_path(self.root, resolved);
                    self.findings.extend(super::super::validate_partial_extends(
                        &available, &relative, self.assertion,
                    ));
                }
                None
            }
            Err(error) => {
                let ancestor_rel = relative_slash_path(self.root, resolved);
                self.findings.push(finding(
                    &ancestor_rel,
                    self.assertion,
                    format!("{ancestor_rel}: {error}"),
                ));
                None
            }
        }
    }
}
