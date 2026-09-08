use super::finding;
use super::keys::Keys;
use super::spec::{extends_specs, is_package_specifier, MAX_EXTENDS_DEPTH};
use crate::codebase::rules::structured_config_policy::paths::canonical_path_in_root;
use crate::codebase::rules::structured_config_policy::ValueAssertion;
use crate::codebase::rules::RuleFinding;
use crate::codebase::structured_value::parse_structured_value;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use serde_yaml::Value;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(super) struct Nested<'a> {
    pub(super) path: &'a Path,
    pub(super) rel: &'a str,
    pub(super) value: &'a Value,
}

pub(super) struct Ancestor {
    pub(super) path: PathBuf,
    pub(super) rel: String,
    pub(super) value: Value,
}

struct Walk<'a> {
    root: &'a Path,
    nested_rel: &'a str,
    sources: &'a SourceStore,
    assertion: &'a ValueAssertion,
    keys: &'a Keys<'a>,
    stack: Vec<PathBuf>,
    seen: HashSet<PathBuf>,
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
) -> Vec<Ancestor> {
    let Some(root) = root.canonicalize().ok() else {
        findings.push(finding(
            nested.rel,
            assertion,
            format!(
                "{}: ancestor-override-subset cannot resolve the repository root safely",
                nested.rel
            ),
        ));
        return Vec::new();
    };
    let Some(nested_path) = canonical_path_in_root(&root, nested.path) else {
        findings.push(finding(
            nested.rel,
            assertion,
            format!(
                "{}: ancestor-override-subset nested config is outside the repository root",
                nested.rel
            ),
        ));
        return Vec::new();
    };
    let mut walk = Walk {
        root: &root,
        nested_rel: nested.rel,
        sources,
        assertion,
        keys,
        stack: vec![nested_path.clone()],
        seen: HashSet::new(),
        ancestors: Vec::new(),
        findings,
    };
    walk.visit(&nested_path, nested.value);
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
        let Some(resolved) = canonical_path_in_root(self.root, &resolved) else {
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
        if !self.seen.insert(resolved.clone()) {
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

    fn load(&mut self, spec: &str, resolved: &Path) -> Option<Value> {
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
        match parse_structured_value(resolved, &source) {
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
