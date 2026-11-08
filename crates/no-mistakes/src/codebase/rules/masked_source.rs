use super::{
    allow_audit::{AllowAudit, CheckedScope},
    path_filter::GlobMatcher,
    RuleFinding,
};
use crate::codebase::ts_source::{has_disable_file_comment, relative_slash_path, SourceStore};
use std::path::Path;

pub(crate) struct MaskedFileContext<'a> {
    pub(crate) root: &'a Path,
    pub(crate) path: &'a Path,
    pub(crate) rule_id: &'a str,
    pub(crate) allow: &'a FileAllow,
    pub(crate) sources: &'a SourceStore,
    pub(crate) defer_suppression: bool,
}

impl<'a> MaskedFileContext<'a> {
    pub(crate) fn new(
        root: &'a Path,
        path: &'a Path,
        rule_id: &'a str,
        allow: &'a FileAllow,
        sources: &'a SourceStore,
        defer_suppression: bool,
    ) -> Self {
        Self {
            root,
            path,
            rule_id,
            allow,
            sources,
            defer_suppression,
        }
    }
}

pub(crate) fn scan_masked_files<T>(
    root: &Path,
    opts: &T,
    files: &[std::path::PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
    check: impl Fn(&Path, &Path, &T, &SourceStore, bool) -> Vec<RuleFinding>,
) -> Vec<RuleFinding> {
    files
        .iter()
        .flat_map(|path| check(root, path, opts, sources, defer_suppression))
        .collect()
}

/// Read, mask, and apply suppression once for file-local Swift/C# scans.
pub(crate) fn scan_masked_file(
    context: MaskedFileContext<'_>,
    mask: fn(&str) -> String,
    detect: impl FnOnce(&str, &str, &str) -> Vec<RuleFinding>,
) -> Vec<RuleFinding> {
    let MaskedFileContext {
        root,
        path,
        rule_id,
        allow,
        sources,
        defer_suppression,
    } = context;
    let rel = relative_slash_path(root, path);
    let Some(source) = super::read_source(sources, path) else {
        return Vec::new();
    };
    let code = mask(&source);
    let mut findings = detect(&source, &code, &rel);
    allow.checked(path);
    if allow.suppress(&rel, !findings.is_empty())
        || (!defer_suppression && has_disable_file_comment(&source, rule_id))
    {
        return Vec::new();
    }
    if !defer_suppression {
        super::suppress_rule_findings_with_source(&mut findings, &source);
    }
    findings
}

pub(crate) struct FileAllow {
    patterns: Vec<String>,
    matchers: Vec<GlobMatcher>,
    audit: Option<AllowAudit>,
    checked: dashmap::DashSet<std::path::PathBuf>,
}

impl FileAllow {
    pub(crate) fn new(patterns: &[String], context: &str) -> anyhow::Result<Self> {
        Ok(Self {
            patterns: patterns.to_vec(),
            matchers: patterns
                .iter()
                .map(|pattern| GlobMatcher::new(std::slice::from_ref(pattern), context))
                .collect::<anyhow::Result<_>>()?,
            audit: None,
            checked: dashmap::DashSet::new(),
        })
    }

    pub(crate) fn setup(
        &mut self,
        root: &Path,
        config: &crate::config::v2::NoMistakesConfig,
        rule: &crate::config::v2::schema::RuleDef,
        sources: &SourceStore,
    ) {
        if self.patterns.is_empty() {
            return;
        }
        self.audit = Some(AllowAudit::new_files(
            root,
            config,
            rule,
            Some(sources),
            "allow",
            self.patterns.len(),
        ));
    }

    pub(crate) fn is_match(&self, rel: &str) -> bool {
        self.matchers.iter().any(|matcher| matcher.is_match(rel))
    }

    pub(crate) fn suppress(&self, rel: &str, has_finding: bool) -> bool {
        if has_finding {
            if let Some(audit) = &self.audit {
                return audit
                    .usage
                    .suppress(self.matchers.iter().map(|matcher| matcher.is_match(rel)));
            }
        }
        self.is_match(rel)
    }

    pub(crate) fn checked(&self, path: &Path) {
        if self.audit.is_some() {
            self.checked.insert(path.to_path_buf());
        }
    }

    pub(crate) fn finish(&self, root: &Path, sources: &SourceStore) -> Vec<RuleFinding> {
        let Some(audit) = &self.audit else {
            return Vec::new();
        };
        let checked: Vec<_> = self.checked.iter().map(|path| path.key().clone()).collect();
        let scope = CheckedScope::new(root, Some(sources), &checked);
        audit.finish_files(
            |index| scope.glob(&self.patterns[index]),
            |index| self.patterns[index].clone(),
            |index| self.patterns[index].clone(),
        )
    }
}
