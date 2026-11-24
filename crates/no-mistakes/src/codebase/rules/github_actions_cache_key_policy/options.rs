use super::text::norm_path;
use super::RULE_ID;
use crate::codebase::check_outcome::CheckInputFailure;
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
    anyhow::ensure!(
        !options.caches.is_empty() || !options.forbidden_paths.is_empty(),
        invalid(
            "caches",
            &options.caches,
            "a non-empty list of caches when forbiddenPaths is empty"
        )
    );
    if let Some(message) = options.forbidden_paths_message.as_deref() {
        anyhow::ensure!(
            !message.trim().is_empty(),
            invalid("forbiddenPathsMessage", message, "a non-empty message")
        );
    }
    let mut seen = BTreeSet::new();
    for cache in &options.caches {
        let path = norm_path(&cache.path);
        anyhow::ensure!(
            !path.is_empty(),
            invalid("caches[].path", &cache.path, "a non-empty path")
        );
        anyhow::ensure!(
            seen.insert(path.clone()),
            invalid("caches[].path", &cache.path, "a unique cache path")
        );
        anyhow::ensure!(
            !norm_path(&cache.lockfile).is_empty(),
            invalid("caches[].lockfile", &cache.lockfile, "a non-empty path")
        );
        anyhow::ensure!(
            !cache.key_parts.is_empty(),
            invalid(
                "caches[].keyParts",
                &cache.key_parts,
                "at least one dotted context fragment"
            )
        );
        for part in &cache.key_parts {
            anyhow::ensure!(
                valid_fragment(part.trim()),
                invalid(
                    "caches[].keyParts",
                    part,
                    "a dotted context fragment such as runner.os"
                )
            );
        }
    }
    let mut forbidden = BTreeSet::new();
    for raw in &options.forbidden_paths {
        let path = norm_path(raw);
        anyhow::ensure!(
            !path.is_empty(),
            invalid("forbiddenPaths", raw, "a non-empty path")
        );
        anyhow::ensure!(
            forbidden.insert(path.clone()),
            invalid("forbiddenPaths", raw, "a unique forbidden path")
        );
        anyhow::ensure!(
            !seen.contains(&path),
            invalid(
                "forbiddenPaths",
                raw,
                "a path absent from options.caches[].path"
            )
        );
    }
    Ok(())
}

fn invalid(option: &str, value: impl std::fmt::Debug, allowed: &str) -> CheckInputFailure {
    CheckInputFailure::invalid_rule_option(RULE_ID, option, value, allowed)
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
