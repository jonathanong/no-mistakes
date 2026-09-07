use super::Ancestor;
use crate::codebase::structured_value::parse_structured_value;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) mod paths;
use paths::{extends, local_specifier};

pub(crate) struct AncestorResolver<'a> {
    root: &'a Path,
    sources: &'a SourceStore,
    values: BTreeMap<PathBuf, Value>,
    chains: BTreeMap<ResolutionKey, Arc<Vec<Ancestor>>>,
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
struct ResolutionKey {
    path: PathBuf,
    extends_key: String,
}

enum Work {
    Enter(PathBuf, Value),
    Exit(PathBuf, PathBuf),
}

impl<'a> AncestorResolver<'a> {
    pub(crate) fn new(root: &'a Path, sources: &'a SourceStore) -> Self {
        Self {
            root,
            sources,
            values: BTreeMap::new(),
            chains: BTreeMap::new(),
        }
    }

    pub(super) fn resolve(
        &mut self,
        path: &Path,
        value: &Value,
        assertion: &super::super::ValueAssertion,
    ) -> Result<Arc<Vec<Ancestor>>, String> {
        let start = normalize_path(path);
        let key = resolution_key(&start, assertion);
        if let Some(chain) = self.chains.get(&key) {
            return Ok(Arc::clone(chain));
        }
        self.values.insert(start.clone(), value.clone());
        let mut direct = BTreeMap::new();
        let mut visiting = BTreeSet::new();
        let mut depth = 0usize;
        let mut work = vec![Work::Enter(start.clone(), value.clone())];
        while let Some(item) = work.pop() {
            match item {
                Work::Enter(path, value) => {
                    depth += 1;
                    if depth > 1024 {
                        return Err(format!(
                            "{}: `{}` exceeds the extends depth limit",
                            relative_slash_path(self.root, &path),
                            assertion.extends_key
                        ));
                    }
                    let key = resolution_key(&path, assertion);
                    if self.chains.contains_key(&key) {
                        continue;
                    }
                    let canonical = path.canonicalize().map_err(|error| {
                        format!(
                            "{}: cannot verify `{}` identity: {error}",
                            relative_slash_path(self.root, &path),
                            assertion.extends_key
                        )
                    })?;
                    if !visiting.insert(canonical.clone()) {
                        return Err(format!(
                            "{}: `{}` contains an extends cycle",
                            relative_slash_path(self.root, &path),
                            assertion.extends_key
                        ));
                    }
                    let parents = self.parents(&path, &value, assertion)?;
                    direct.insert(path.clone(), parents.clone());
                    work.push(Work::Exit(path, canonical));
                    for parent in parents.into_iter().rev() {
                        if !self
                            .chains
                            .contains_key(&resolution_key(&parent, assertion))
                        {
                            work.push(Work::Enter(
                                parent.clone(),
                                self.values.get(&parent).cloned().expect("loaded parent"),
                            ));
                        }
                    }
                }
                Work::Exit(path, canonical) => {
                    visiting.remove(&canonical);
                    let mut chain = Vec::new();
                    for parent in direct.get(&path).into_iter().flatten() {
                        chain.extend(
                            self.chains
                                .get(&resolution_key(parent, assertion))
                                .expect("parents resolve before children")
                                .iter()
                                .cloned(),
                        );
                        chain.push(Ancestor {
                            path: parent.clone(),
                            value: self.values.get(parent).cloned().expect("loaded parent"),
                        });
                    }
                    self.chains
                        .insert(resolution_key(&path, assertion), Arc::new(chain));
                }
            }
        }
        Ok(self
            .chains
            .get(&key)
            .map(Arc::clone)
            .expect("start chain resolved"))
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

    fn ensure_contained(&self, from: &Path, candidate: &Path, key: &str) -> Result<(), String> {
        let contained =
            super::super::path_containment::verify(self.root, candidate).map_err(|error| {
                format!(
                    "{}: cannot verify `{key}` repository containment: {error}",
                    relative_slash_path(self.root, from)
                )
            })?;
        if contained {
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
        let rel = relative_slash_path(self.root, path);
        let source = super::super::super::read_source(self.sources, path)
            .ok_or_else(|| format!("{rel}: extends reference is missing"))?;
        let value =
            parse_structured_value(path, &source).map_err(|error| format!("{rel}: {error}"))?;
        self.values.insert(path.to_path_buf(), value);
        Ok(())
    }
}

fn resolution_key(path: &Path, assertion: &super::super::ValueAssertion) -> ResolutionKey {
    ResolutionKey {
        path: path.to_path_buf(),
        extends_key: assertion.extends_key.clone(),
    }
}
