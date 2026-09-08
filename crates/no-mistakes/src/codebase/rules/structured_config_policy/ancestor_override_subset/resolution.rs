use super::Ancestor;
use crate::codebase::structured_value::parse_structured_value;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(super) mod paths;
use paths::{extends, local_specifier};

pub(crate) struct AncestorResolver<'a> {
    root: &'a Path,
    root_identity: Result<PathBuf, String>,
    sources: &'a SourceStore,
    values: BTreeMap<PathBuf, Value>,
    values_by_identity: BTreeMap<PathBuf, Value>,
    identities: BTreeMap<PathBuf, Result<PathBuf, String>>,
}

impl<'a> AncestorResolver<'a> {
    pub(crate) fn new(root: &'a Path, sources: &'a SourceStore, candidates: &[PathBuf]) -> Self {
        let root_identity = root.canonicalize().map_err(|error| error.to_string());
        let identities = candidates
            .iter()
            .map(|path| {
                let path = normalize_path(path);
                let identity = path.canonicalize().map_err(|error| error.to_string());
                (path, identity)
            })
            .collect();
        Self {
            root,
            root_identity,
            sources,
            values: BTreeMap::new(),
            values_by_identity: BTreeMap::new(),
            identities,
        }
    }

    pub(super) fn resolve(
        &mut self,
        path: &Path,
        value: &Value,
        assertion: &super::super::ValueAssertion,
    ) -> Result<Vec<Ancestor>, String> {
        let start = normalize_path(path);
        self.values.insert(start.clone(), value.clone());
        let mut active = BTreeSet::new();
        self.resolve_from(&start, value, assertion, &mut active, 0)
    }

    fn resolve_from(
        &mut self,
        path: &Path,
        value: &Value,
        assertion: &super::super::ValueAssertion,
        active: &mut BTreeSet<PathBuf>,
        depth: usize,
    ) -> Result<Vec<Ancestor>, String> {
        if depth >= 1024 {
            return Err(format!(
                "{}: `{}` exceeds the extends depth limit",
                relative_slash_path(self.root, path),
                assertion.extends_key
            ));
        }
        let identity = self.identity(path, &assertion.extends_key)?;
        if !active.insert(identity.clone()) {
            return Err(format!(
                "{}: `{}` contains an extends cycle",
                relative_slash_path(self.root, path),
                assertion.extends_key
            ));
        }
        let result = (|| {
            let mut chain = Vec::new();
            for parent in self.parents(path, value, assertion)? {
                let parent_value = self.values.get(&parent).cloned().expect("loaded parent");
                chain.extend(self.resolve_from(
                    &parent,
                    &parent_value,
                    assertion,
                    active,
                    depth + 1,
                )?);
                chain.push(Ancestor {
                    path: parent,
                    value: parent_value,
                });
            }
            Ok(chain)
        })();
        active.remove(&identity);
        result
    }

    fn parents(
        &mut self,
        path: &Path,
        value: &Value,
        assertion: &super::super::ValueAssertion,
    ) -> Result<Vec<PathBuf>, String> {
        let mut parents = Vec::new();
        for specifier in extends(value, &assertion.extends_key)? {
            let Some(specifier) = local_specifier(&specifier, &assertion.extends_key)? else {
                continue;
            };
            let parent = normalize_path(&path.parent().unwrap_or(self.root).join(specifier));
            if !parent.exists() {
                return Err(format!(
                    "{}: `{}` reference is missing",
                    relative_slash_path(self.root, path),
                    assertion.extends_key
                ));
            }
            self.ensure_contained(path, &parent, &assertion.extends_key)?;
            self.load(&parent)?;
            parents.push(parent);
        }
        Ok(parents)
    }

    fn ensure_contained(&mut self, from: &Path, candidate: &Path, key: &str) -> Result<(), String> {
        let root = self.root_identity.clone().map_err(|error| {
            format!(
                "{}: cannot verify `{key}` repository containment: {error}",
                relative_slash_path(self.root, from)
            )
        })?;
        let candidate = self.identity(candidate, key).map_err(|error| {
            format!(
                "{}: cannot verify `{key}` repository containment: {error}",
                relative_slash_path(self.root, from)
            )
        })?;
        if candidate.strip_prefix(&root).is_ok() {
            return Ok(());
        }
        Err(format!(
            "{}: `{key}` reference is outside the repository root",
            relative_slash_path(self.root, from)
        ))
    }

    fn load(&mut self, path: &Path) -> Result<(), String> {
        if self.values.contains_key(path) {
            return Ok(());
        }
        let identity = self.identity(path, "extends")?;
        if let Some(value) = self.values_by_identity.get(&identity) {
            self.values.insert(path.to_path_buf(), value.clone());
            return Ok(());
        }
        let rel = relative_slash_path(self.root, path);
        let source = super::super::super::read_source(self.sources, path)
            .ok_or_else(|| format!("{rel}: extends reference is missing"))?;
        let value =
            parse_structured_value(path, &source).map_err(|error| format!("{rel}: {error}"))?;
        self.values.insert(path.to_path_buf(), value.clone());
        self.values_by_identity.insert(identity, value);
        Ok(())
    }

    fn identity(&mut self, path: &Path, key: &str) -> Result<PathBuf, String> {
        let path = normalize_path(path);
        let identity = self
            .identities
            .entry(path.clone())
            .or_insert_with(|| path.canonicalize().map_err(|error| error.to_string()));
        identity.clone().map_err(|error| {
            format!(
                "{}: cannot verify `{key}` identity: {error}",
                relative_slash_path(self.root, &path)
            )
        })
    }

    #[cfg(test)]
    pub(crate) fn resolve_for_test(
        &mut self,
        path: &Path,
        value: &Value,
        assertion: &super::super::ValueAssertion,
    ) -> Result<Vec<Ancestor>, String> {
        self.resolve(path, value, assertion)
    }

    #[cfg(test)]
    pub(crate) fn cached_identity_count(&self) -> usize {
        self.identities.len()
    }
}
