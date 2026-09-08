use super::finding;
use super::keys::Keys;
use super::spec::{
    extends_specs, is_package_specifier, MAX_EXTENDS_DEPTH, MAX_EXTENDS_OCCURRENCES,
};
use crate::codebase::rules::structured_config_policy::paths::canonical_path_in_canonical_root;
use crate::codebase::rules::structured_config_policy::ValueAssertion;
use crate::codebase::rules::RuleFinding;
use crate::codebase::structured_value::parse_structured_value;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use serde_yaml::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) struct Nested<'a> {
    pub(super) path: &'a Path,
    pub(super) rel: &'a str,
    pub(super) value: &'a Value,
}

pub(super) struct Ancestor {
    pub(super) path: PathBuf,
    pub(super) rel: String,
    pub(super) value: Arc<Value>,
}

#[derive(Default)]
pub(in crate::codebase::rules::structured_config_policy) struct ParsedAncestorCache {
    values: BTreeMap<PathBuf, Result<Arc<Value>, String>>,
    #[cfg(test)]
    parse_counts: BTreeMap<PathBuf, usize>,
}

impl ParsedAncestorCache {
    fn parse(&mut self, path: &Path, source: &str) -> Result<Arc<Value>, String> {
        if let Some(value) = self.values.get(path) {
            return value.clone();
        }
        #[cfg(test)]
        {
            *self.parse_counts.entry(path.to_path_buf()).or_default() += 1;
        }
        let value = parse_structured_value(path, source)
            .map(Arc::new)
            .map_err(|error| error.to_string());
        self.values.insert(path.to_path_buf(), value.clone());
        value
    }

    #[cfg(test)]
    pub(in crate::codebase::rules::structured_config_policy) fn parse_count(
        &self,
        path: &Path,
    ) -> usize {
        self.parse_counts.get(path).copied().unwrap_or_default()
    }
}

struct Walk<'a> {
    root: &'a Path,
    nested_rel: &'a str,
    sources: &'a SourceStore,
    assertion: &'a ValueAssertion,
    keys: &'a Keys<'a>,
    stack: Vec<PathBuf>,
    occurrences: usize,
    max_occurrences: usize,
    traversal_exhausted: bool,
    parsed_ancestors: &'a mut ParsedAncestorCache,
    ancestors: Vec<Ancestor>,
    findings: &'a mut Vec<RuleFinding>,
}

pub(super) fn collect_ancestors(
    root: &Path,
    nested: Nested<'_>,
    sources: &SourceStore,
    assertion: &ValueAssertion,
    keys: &Keys<'_>,
    findings: &mut Vec<RuleFinding>,
    parsed_ancestors: &mut ParsedAncestorCache,
) -> Vec<Ancestor> {
    let mut walk = Walk {
        root,
        nested_rel: nested.rel,
        sources,
        assertion,
        keys,
        stack: vec![nested.path.to_path_buf()],
        occurrences: 0,
        max_occurrences: MAX_EXTENDS_OCCURRENCES,
        traversal_exhausted: false,
        parsed_ancestors,
        ancestors: Vec::new(),
        findings,
    };
    walk.visit(nested.path, nested.value);
    walk.ancestors
}

impl Walk<'_> {
    fn visit(&mut self, from_path: &Path, from_value: &Value) {
        let from_dir = from_path.parent().unwrap_or(from_path);
        for spec in self.extends(from_value) {
            self.follow(from_dir, spec);
        }
    }

    fn extends<'b>(&mut self, value: &'b Value) -> Vec<&'b str> {
        match extends_specs(value, self.keys.extends) {
            Ok(specs) => specs,
            Err(detail) => {
                self.invalid_extends(detail);
                Vec::new()
            }
        }
    }

    fn invalid_extends(&mut self, detail: &str) {
        self.findings.push(finding(
            self.nested_rel,
            self.assertion,
            format!(
                "{}: ancestor-override-subset extends {detail}",
                self.nested_rel
            ),
        ));
    }

    fn follow(&mut self, from_dir: &Path, spec: &str) {
        if is_package_specifier(spec) {
            return;
        }
        if self.traversal_exhausted {
            return;
        }
        if self.occurrences >= self.max_occurrences {
            self.traversal_exhausted = true;
            self.findings.push(finding(
                self.nested_rel,
                self.assertion,
                format!(
                    "{}: ancestor-override-subset extends traversal exceeds the maximum of {} occurrences",
                    self.nested_rel, self.max_occurrences
                ),
            ));
            return;
        }
        self.occurrences += 1;
        let resolved = normalize_path(&from_dir.join(spec));
        if !resolved.exists() {
            self.findings.push(finding(
                self.nested_rel,
                self.assertion,
                format!(
                    "{}: ancestor-override-subset `{spec}` is missing",
                    self.nested_rel
                ),
            ));
            return;
        }
        let Some(resolved) = canonical_path_in_canonical_root(self.root, &resolved) else {
            self.findings.push(finding(
                self.nested_rel,
                self.assertion,
                format!(
                    "{}: ancestor-override-subset `{spec}` is outside the repository root",
                    self.nested_rel
                ),
            ));
            return;
        };
        if self.stack.len() >= MAX_EXTENDS_DEPTH {
            self.findings.push(finding(
                self.nested_rel,
                self.assertion,
                format!(
                    "{}: ancestor-override-subset extends chain exceeds the maximum depth of {MAX_EXTENDS_DEPTH}",
                    self.nested_rel
                ),
            ));
            return;
        }
        if self.stack.iter().any(|path| path == &resolved) {
            self.findings.push(finding(
                self.nested_rel,
                self.assertion,
                format!(
                    "{}: ancestor-override-subset extends cycle through `{spec}`",
                    self.nested_rel
                ),
            ));
            return;
        }
        let Some(value) = self.load(spec, &resolved) else {
            return;
        };
        self.stack.push(resolved.clone());
        self.visit(&resolved, &value);
        self.stack.pop();
        self.ancestors.push(Ancestor {
            rel: relative_slash_path(self.root, &resolved),
            path: resolved,
            value,
        });
    }

    fn load(&mut self, spec: &str, resolved: &Path) -> Option<Arc<Value>> {
        let Some(source) = crate::codebase::rules::read_source(self.sources, resolved) else {
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

#[cfg(test)]
mod tests;
