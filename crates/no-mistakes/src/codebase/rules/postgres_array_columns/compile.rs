use super::Options;
use crate::codebase::postgres::{require_catalog_path, AllowList};
use anyhow::{bail, Result};

pub(super) struct Compiled {
    pub(super) never: Vec<String>,
    pub(super) allow_types: Vec<String>,
    pub(super) allow_enums: bool,
    pub(super) allow: AllowList,
    pub(super) message: Option<String>,
}

impl Compiled {
    pub(super) fn listed(types: &[String], element: &str) -> bool {
        types
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(element))
    }
}

pub(super) fn compile(options: &Options, message: Option<String>) -> Result<Compiled> {
    require_catalog_path(super::RULE_ID, options.schema_catalog_path.trim())?;
    let overlap: Vec<&str> = options
        .allow_element_types
        .iter()
        .filter(|element| Compiled::listed(&options.never_allow_element_types, element))
        .map(String::as_str)
        .collect();
    if !overlap.is_empty() {
        bail!(
            "{} option allowElementTypes: overlaps neverAllowElementTypes: {}",
            super::RULE_ID,
            overlap.join(", ")
        );
    }
    Ok(Compiled {
        never: options.never_allow_element_types.clone(),
        allow_types: options.allow_element_types.clone(),
        allow_enums: options.allow_enum_elements,
        allow: AllowList::compile(super::RULE_ID, options.allow.clone())?,
        message,
    })
}
