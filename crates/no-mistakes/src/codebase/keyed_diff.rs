use super::swift::parse_resolved_pins;
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod catalogs;
mod encoding;
mod nuget;
mod strings;

pub(crate) use nuget::nuget_entries;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum KeyFormat {
    #[serde(alias = "pnpm-lock")]
    Pnpm,
    #[serde(alias = "package-lock")]
    Npm,
    #[serde(alias = "swiftpm")]
    Swift,
    Nuget,
    Json,
    Strings,
    Resx,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KeyOptions {
    pub format: KeyFormat,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub array: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id_field: Option<String>,
}

impl KeyOptions {
    pub fn validate(&self) -> Result<()> {
        match (&self.array, &self.id_field) {
            (None, None) => {}
            (Some(array), Some(id_field)) if self.format == KeyFormat::Json => {
                ensure!(!array.is_empty(), "JSON array path must be nonempty");
                ensure!(!id_field.is_empty(), "JSON array ID field must be nonempty");
            }
            _ => anyhow::bail!("array and idField must be supplied together and require JSON"),
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyChanges {
    #[serde(default)]
    pub changed: Vec<String>,
    #[serde(default)]
    pub added: Vec<String>,
    #[serde(default)]
    pub removed: Vec<String>,
}

impl KeyChanges {
    pub fn is_empty(&self) -> bool {
        self.changed.is_empty() && self.added.is_empty() && self.removed.is_empty()
    }

    pub fn all_ids(self) -> impl Iterator<Item = String> {
        self.changed
            .into_iter()
            .chain(self.added)
            .chain(self.removed)
    }
}

pub fn diff(options: &KeyOptions, old: Option<&[u8]>, new: Option<&[u8]>) -> Result<KeyChanges> {
    diff_with_presence(options, old, new).map(|(changes, _)| changes)
}

pub(crate) fn diff_with_presence(
    options: &KeyOptions,
    old: Option<&[u8]>,
    new: Option<&[u8]>,
) -> Result<(KeyChanges, [Vec<String>; 2])> {
    options.validate()?;
    let format = options.format;
    match format {
        KeyFormat::Pnpm => (super::package_analysis::hooks().diff_pnpm_keys)(old, new),
        KeyFormat::Npm => (super::package_analysis::hooks().diff_npm_keys)(old, new),
        KeyFormat::Swift => diff_parsed(format, old, new, |source| {
            let pins = parse_resolved_pins(source)
                .map_err(|error| anyhow::anyhow!("invalid Swift lockfile: {error:?}"))?;
            Ok(pins
                .into_iter()
                .map(|pin| (pin.identity.clone(), pin))
                .collect())
        }),
        KeyFormat::Nuget => diff_parsed(format, old, new, |source| {
            let root = serde_json::from_str(source).context("invalid NuGet lockfile JSON")?;
            nuget_entries(&root).map_err(|_| anyhow::anyhow!("unsupported NuGet lockfile schema"))
        }),
        KeyFormat::Json => diff_parsed(format, old, new, |source| {
            catalogs::json_entries(source, options)
        }),
        KeyFormat::Strings => diff_parsed(format, old, new, strings::entries),
        KeyFormat::Resx => diff_parsed(format, old, new, catalogs::resx_entries),
    }
}

pub(super) fn diff_parsed<V: PartialEq>(
    format: KeyFormat,
    old: Option<&[u8]>,
    new: Option<&[u8]>,
    parse: impl Fn(&str) -> Result<BTreeMap<String, V>>,
) -> Result<(KeyChanges, [Vec<String>; 2])> {
    let parse = |bytes: &[u8]| parse(&encoding::decode_input(format, bytes)?);
    let old_entries = parse_side(old, "base", parse)?.unwrap_or_default();
    let old_keys: Vec<_> = old_entries.keys().cloned().collect();
    if old == new {
        return Ok((KeyChanges::default(), [old_keys.clone(), old_keys]));
    }
    let new_entries = parse_side(new, "head", parse)?.unwrap_or_default();
    let changes = diff_entries(&old_entries, &new_entries);
    Ok((changes, [old_keys, new_entries.into_keys().collect()]))
}

fn parse_side<T>(
    source: Option<&[u8]>,
    side: &str,
    parse: impl FnOnce(&[u8]) -> Result<T>,
) -> Result<Option<T>> {
    source
        .map(parse)
        .transpose()
        .with_context(|| format!("could not parse {side} content"))
}

pub fn diff_entries<K: Ord + AsRef<str>, V: PartialEq>(
    old: &BTreeMap<K, V>,
    new: &BTreeMap<K, V>,
) -> KeyChanges {
    let mut changes = KeyChanges::default();
    for (key, value) in old {
        match new.get(key) {
            None => changes.removed.push(key.as_ref().to_string()),
            Some(new) if value != new => changes.changed.push(key.as_ref().to_string()),
            _ => {}
        }
    }
    for key in new.keys().filter(|key| !old.contains_key(*key)) {
        changes.added.push(key.as_ref().to_string());
    }
    changes
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;
