use super::text::norm_path;
use super::RULE_ID;
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
    pub caches: Vec<CacheSpec>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            reject_restore_keys: true,
            allow_hash_files: false,
            caches: Vec::new(),
        }
    }
}

pub(super) fn validate(options: &Options) -> Result<()> {
    anyhow::ensure!(
        !options.caches.is_empty(),
        "{RULE_ID} caches must list at least one cache"
    );
    let mut seen = BTreeSet::new();
    for cache in &options.caches {
        let path = norm_path(&cache.path);
        anyhow::ensure!(
            !path.is_empty(),
            "{RULE_ID} caches.path must be a non-empty path"
        );
        anyhow::ensure!(
            seen.insert(path.clone()),
            "{RULE_ID} caches.path `{path}` is listed more than once"
        );
        anyhow::ensure!(
            !norm_path(&cache.lockfile).is_empty(),
            "{RULE_ID} caches.lockfile must be a non-empty path"
        );
        anyhow::ensure!(
            !cache.key_parts.is_empty(),
            "{RULE_ID} caches.keyParts must list at least one expression fragment"
        );
        for part in &cache.key_parts {
            anyhow::ensure!(
                valid_fragment(part.trim()),
                "{RULE_ID} caches.keyParts entries must be dotted context fragments such as runner.os"
            );
        }
    }
    Ok(())
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
