use super::{Options, RULE_ID};
use crate::codebase::postgres::{
    require_catalog_path, AllowEntry, AllowList, CatalogObjectRef,
};
use anyhow::{bail, Result};

pub(in crate::codebase::rules) struct Compiled {
    pub(super) type_suffix: String,
    pub(super) id_suffix: String,
    pub(super) allow: AllowList,
    pub(super) message: Option<String>,
}

pub(in crate::codebase::rules) fn compile(
    options: &Options,
    message: Option<String>,
) -> Result<Compiled> {
    require_catalog_path(RULE_ID, &options.schema_catalog_path)?;
    if options.type_suffix.trim().is_empty() {
        bail!("{RULE_ID} option typeSuffix: must not be empty");
    }
    if options.id_suffix.trim().is_empty() {
        bail!("{RULE_ID} option idSuffix: must not be empty");
    }
    if options.type_suffix == options.id_suffix {
        bail!("{RULE_ID} options: typeSuffix and idSuffix must be different");
    }
    let entries = options
        .allow
        .iter()
        .map(|entry| {
            let object = format!("column:{}", entry.column);
            if !matches!(
                object.parse::<CatalogObjectRef>(),
                Ok(CatalogObjectRef::Column { .. })
            ) {
                bail!(
                    "{RULE_ID} options.allow.column: expected table.column, got {}",
                    entry.column
                );
            }
            Ok(AllowEntry {
                object,
                reason: entry.reason.clone(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Compiled {
        type_suffix: options.type_suffix.clone(),
        id_suffix: options.id_suffix.clone(),
        allow: AllowList::compile(RULE_ID, entries)?,
        message: message.filter(|text| !text.trim().is_empty()),
    })
}
