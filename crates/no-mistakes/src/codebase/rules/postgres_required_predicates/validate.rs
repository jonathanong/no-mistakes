use super::{AllowEntry, Options, PartitionExemption, RULE_ID};
use anyhow::{bail, Result};
use std::collections::BTreeSet;

pub(super) fn check(opts: &Options) -> Result<()> {
    for relation in &opts.relations {
        if relation
            .require_columns
            .iter()
            .any(|column| column.trim().is_empty())
        {
            bail!("{RULE_ID} option requireColumns: empty column name");
        }
    }
    match opts.partition_keys.as_str() {
        "" | "off" => {}
        "require" => {
            if opts.schema_catalog_path.trim().is_empty() {
                bail!("{RULE_ID} option schemaCatalogPath: required when partitionKeys is require");
            }
        }
        _ => bail!("{RULE_ID} option partitionKeys: expected require or off"),
    }
    distinct_tables(&opts.partition_key_exemptions)?;
    distinct_allows(&opts.allow)
}

fn distinct_tables(entries: &[PartitionExemption]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for entry in entries {
        if entry.table.trim().is_empty() {
            bail!("{RULE_ID} option partitionKeyExemptions: empty table name");
        }
        if entry.reason.trim().is_empty() {
            bail!("{RULE_ID} option partitionKeyExemptions: empty reason");
        }
        if !seen.insert(entry.table.to_ascii_lowercase()) {
            bail!(
                "{RULE_ID} option partitionKeyExemptions: duplicate entry {}",
                entry.table
            );
        }
    }
    Ok(())
}

fn distinct_allows(entries: &[AllowEntry]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for entry in entries {
        if entry.object.trim().is_empty() {
            bail!("{RULE_ID} option allow: empty object");
        }
        if entry.reason.trim().is_empty() {
            bail!("{RULE_ID} option allow: empty reason");
        }
        if !seen.insert(entry.object.clone()) {
            bail!("{RULE_ID} option allow: duplicate entry {}", entry.object);
        }
    }
    Ok(())
}
