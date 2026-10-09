use super::Options;
use crate::codebase::postgres::{require_catalog_path, AllowList, CatalogObjectRef};
use anyhow::{bail, Result};
use std::collections::BTreeSet;

pub(super) struct Compiled {
    pub(super) allowed_types: Vec<String>,
    pub(super) allow_enum_types: bool,
    pub(super) check_primary_keys: bool,
    pub(super) check_foreign_keys: bool,
    pub(super) allow: AllowList,
    pub(super) message: Option<String>,
}

pub(super) fn compile(options: &Options, message: Option<String>) -> Result<Compiled> {
    require_catalog_path(super::RULE_ID, options.schema_catalog_path.trim())?;
    if options.allowed_types.is_empty() {
        bail!(
            "{} option allowedTypes: required and nonempty",
            super::RULE_ID
        );
    }
    let mut seen = BTreeSet::new();
    for data_type in &options.allowed_types {
        let value = data_type.trim();
        if value.is_empty() {
            bail!(
                "{} option allowedTypes: entries must be nonempty",
                super::RULE_ID
            );
        }
        if !seen.insert(value.to_ascii_lowercase()) {
            bail!(
                "{} option allowedTypes: duplicate entry {value}",
                super::RULE_ID
            );
        }
    }
    if !options.check_primary_keys && !options.check_foreign_keys {
        bail!(
            "{} option checkPrimaryKeys/checkForeignKeys: at least one must be true",
            super::RULE_ID
        );
    }
    for entry in &options.allow {
        match entry.object.parse::<CatalogObjectRef>() {
            Ok(CatalogObjectRef::Constraint { .. }) => {}
            Ok(_) => bail!(
                "{} option allow: expected a constraint object ref, got {}",
                super::RULE_ID,
                entry.object
            ),
            Err(_) => {}
        }
    }
    Ok(Compiled {
        allowed_types: options
            .allowed_types
            .iter()
            .map(|value| value.trim().to_string())
            .collect(),
        allow_enum_types: options.allow_enum_types,
        check_primary_keys: options.check_primary_keys,
        check_foreign_keys: options.check_foreign_keys,
        allow: AllowList::compile(super::RULE_ID, options.allow.clone())?,
        message,
    })
}
