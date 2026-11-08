use super::{Options, RULE_ID};
use crate::codebase::postgres::{AllowEntry, AllowList, CatalogObjectRef};
use crate::codebase::rules::path_filter::GlobMatcher;
use anyhow::{bail, Result};

pub(super) struct Compiled {
    pub(super) requirements: Vec<Requirement>,
    pub(super) allow: AllowList,
    pub(super) message: Option<String>,
}

pub(super) struct Requirement {
    pub(super) pattern: GlobMatcher,
    pub(super) types: Vec<&'static str>,
}

pub(super) fn compile(options: &Options, message: Option<String>) -> Result<Compiled> {
    let requirements = options.columns.iter().map(|requirement| {
        if requirement.pattern.trim().is_empty() || requirement.types.is_empty() {
            bail!("{RULE_ID} options.columns: pattern and types must be nonempty");
        }
        let types = requirement.types.iter().map(|data_type| {
            integer_type(data_type).ok_or_else(|| anyhow::anyhow!(
                "{RULE_ID} options.columns.types: unsupported integer type {data_type}; use smallint, integer, or bigint"
            ))
        }).collect::<Result<Vec<_>>>()?;
        Ok(Requirement {
            pattern: GlobMatcher::new(std::slice::from_ref(&requirement.pattern), RULE_ID)?,
            types,
        })
    }).collect::<Result<Vec<_>>>()?;
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
        requirements,
        allow: AllowList::compile(RULE_ID, entries)?,
        message,
    })
}

pub(super) fn integer_type(data_type: &str) -> Option<&'static str> {
    match data_type.trim().to_ascii_lowercase().as_str() {
        "smallint" | "int2" => Some("smallint"),
        "integer" | "int" | "int4" => Some("integer"),
        "bigint" | "int8" => Some("bigint"),
        _ => None,
    }
}
