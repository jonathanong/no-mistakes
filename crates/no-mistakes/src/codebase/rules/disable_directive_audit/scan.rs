use super::known_rules::known_rule;
use crate::codebase::rules::source_access::read_failure;
use crate::codebase::rules::suppression::{SuppressionDirective, SuppressionDirectiveKind};
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::{
    audit_directives_for_path, relative_slash_path, DisableDirective, SourceStore,
};
use rayon::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(super) const UNUSED_DISABLE_DIRECTIVE: &str = "unused-disable-directive";
pub(super) const INVALID_DISABLE_DIRECTIVE: &str = "invalid-disable-directive";

pub(super) struct PreparedComment {
    pub(super) file: String,
    pub(super) rule: String,
    pub(super) directive: SuppressionDirective,
    pub(super) status: DirectiveStatus,
}

pub(super) enum DirectiveStatus {
    Enabled,
    Disabled,
    Unknown,
    IneffectivePlacement,
}

pub(super) fn scan_scopes(
    root: &Path,
    scopes: &BTreeMap<PathBuf, BTreeSet<&str>>,
    sources: &SourceStore,
    audit_hygiene: bool,
) -> (Vec<PreparedComment>, Vec<RuleFinding>) {
    let per_file: Vec<_> = scopes
        .par_iter()
        .map(|(path, enabled)| {
            let source = match sources.read_source_path(path) {
                Ok(source) => source,
                Err(error) => {
                    return (
                        Vec::new(),
                        audit_hygiene
                            .then(|| read_failure(root, path, UNUSED_DISABLE_DIRECTIVE, &error)),
                    );
                }
            };
            let file = relative_slash_path(root, path);
            let comments = audit_directives_for_path(&source, path)
                .into_iter()
                .filter_map(|(named, ineffective)| {
                    let status = if audit_hygiene && ineffective {
                        DirectiveStatus::IneffectivePlacement
                    } else if !known_rule(&named.rule) {
                        DirectiveStatus::Unknown
                    } else if !audit_hygiene {
                        return None;
                    } else if enabled.contains(named.rule.as_str()) {
                        DirectiveStatus::Enabled
                    } else {
                        DirectiveStatus::Disabled
                    };
                    Some(PreparedComment {
                        file: file.clone(),
                        rule: named.rule,
                        directive: project(named.directive),
                        status,
                    })
                })
                .collect();
            (comments, None)
        })
        .collect();
    let mut comments = Vec::new();
    let mut read_failures = Vec::new();
    for (file_comments, failure) in per_file {
        comments.extend(file_comments);
        if let Some(failure) = failure {
            read_failures.push(failure);
        }
    }
    (comments, read_failures)
}

pub(super) fn project(directive: DisableDirective) -> SuppressionDirective {
    let (kind, line) = match directive {
        DisableDirective::File { line } => (SuppressionDirectiveKind::File, line),
        DisableDirective::Line { line } => (SuppressionDirectiveKind::Line, line),
        DisableDirective::NextLine { line } => (SuppressionDirectiveKind::NextLine, line),
    };
    SuppressionDirective {
        kind,
        line: line as usize,
    }
}
