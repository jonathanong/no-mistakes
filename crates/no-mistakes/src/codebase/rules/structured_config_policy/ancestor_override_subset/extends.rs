use super::finding;
use super::keys::Keys;
use super::spec::{
    MAX_EXTENDS_DEPTH, MAX_EXTENDS_OCCURRENCES, extends_specs, is_package_specifier,
};
use crate::codebase::rules::file_diagnostics::ClassifiedInputFindings;
use crate::codebase::rules::structured_config_policy::ValueAssertion;
use crate::codebase::rules::structured_config_policy::paths::canonical_path_in_canonical_root;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::{SourceStore, relative_slash_path};
use serde_yaml::Value;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod cache;
pub(in crate::codebase::rules::structured_config_policy) use cache::ParsedAncestorCache;

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
    result: &'a mut ClassifiedInputFindings,
}

pub(super) fn collect_ancestors(
    root: &Path,
    nested: Nested<'_>,
    sources: &SourceStore,
    assertion: &ValueAssertion,
    keys: &Keys<'_>,
    result: &mut ClassifiedInputFindings,
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
        result,
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
        self.result.findings.push(finding(
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
            self.result.findings.push(finding(
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
            self.result.findings.push(finding(
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
            self.result.findings.push(finding(
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
            self.result.findings.push(finding(
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
            self.result.findings.push(finding(
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
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
