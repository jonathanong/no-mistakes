use super::text::norm_path;
use super::RULE_ID;
use crate::codebase::rules::validate_options::{finish, format_invalid_option};
use anyhow::Result;
use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct CacheSpec {
    pub path: String,
    pub lockfile: String,
    pub key_parts: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct Options {
    pub reject_restore_keys: bool,
    pub allow_hash_files: bool,
    pub forbid_broad_hash_files: bool,
    pub allowed_hash_files_patterns: Vec<String>,
    pub forbidden_paths: Vec<String>,
    pub forbidden_paths_message: Option<String>,
    pub caches: Vec<CacheSpec>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            reject_restore_keys: true,
            allow_hash_files: false,
            forbid_broad_hash_files: false,
            allowed_hash_files_patterns: Vec::new(),
            forbidden_paths: Vec::new(),
            forbidden_paths_message: None,
            caches: Vec::new(),
        }
    }
}

pub(super) fn validate(options: &Options) -> Result<()> {
    let mut errors = BTreeSet::new();
    collect_errors(options, &mut errors);
    finish(errors.into_iter().collect())
}

pub(super) fn collect_errors(options: &Options, errors: &mut BTreeSet<String>) {
    if options.caches.is_empty() && options.forbidden_paths.is_empty() {
        errors.insert(invalid(
            "caches",
            &options.caches,
            "a non-empty list of caches when forbiddenPaths is empty",
        ));
    }
    if let Some(message) = options.forbidden_paths_message.as_deref() {
        if message.trim().is_empty() {
            errors.insert(invalid(
                "forbiddenPathsMessage",
                message,
                "a non-empty message",
            ));
        }
    }
    let mut seen = BTreeSet::new();
    for cache in &options.caches {
        let path = norm_path(&cache.path);
        if path.is_empty() {
            errors.insert(invalid("caches[].path", &cache.path, "a non-empty path"));
        } else if !seen.insert(path) {
            errors.insert(invalid(
                "caches[].path",
                &cache.path,
                "a unique cache path",
            ));
        }
        if norm_path(&cache.lockfile).is_empty() {
            errors.insert(invalid(
                "caches[].lockfile",
                &cache.lockfile,
                "a non-empty path",
            ));
        }
        if cache.key_parts.is_empty() {
            errors.insert(invalid(
                "caches[].keyParts",
                &cache.key_parts,
                "at least one dotted context fragment",
            ));
        } else {
            for part in &cache.key_parts {
                if !valid_fragment(part.trim()) {
                    errors.insert(invalid(
                        "caches[].keyParts",
                        part,
                        "a dotted context fragment such as runner.os",
                    ));
                }
            }
        }
    }
    let mut forbidden = BTreeSet::new();
    for raw in &options.forbidden_paths {
        let path = norm_path(raw);
        if path.is_empty() {
            errors.insert(invalid("forbiddenPaths", raw, "a non-empty path"));
            continue;
        }
        if !forbidden.insert(path.clone()) {
            errors.insert(invalid("forbiddenPaths", raw, "a unique forbidden path"));
        }
        if seen.contains(&path) {
            errors.insert(invalid(
                "forbiddenPaths",
                raw,
                "a path absent from options.caches[].path",
            ));
        }
    }
}

fn invalid(option: &str, value: impl std::fmt::Debug, allowed: &str) -> String {
    format_invalid_option(RULE_ID, option, value, allowed)
}

fn valid_fragment(part: &str) -> bool {
    let mut pieces = part.split('.');
    let (Some(first), Some(second)) = (pieces.next(), pieces.next()) else {
        return false;
    };
    ident(first) && ident(second) && pieces.all(ident)
}

fn ident(part: &str) -> bool {
    let mut chars = part.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
