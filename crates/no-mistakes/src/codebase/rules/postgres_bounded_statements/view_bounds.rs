//! An explicit, request-scoped cardinality contract for catalog views.
use super::RULE_ID;
use crate::codebase::postgres::{CatalogView, SchemaCatalog};
use anyhow::{bail, Result};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(super) enum ViewBoundMode {
    Bounded,
    Unbounded,
}

#[derive(Default)]
pub(super) struct ViewBounds(BTreeMap<String, ViewBoundMode>);

impl ViewBounds {
    pub(super) fn compile(
        configured: &BTreeMap<String, ViewBoundMode>,
        catalog: &SchemaCatalog,
    ) -> Result<Self> {
        let mut resolved = BTreeMap::new();
        for (name, mode) in configured {
            let Some(view) = catalog.bounded_view_relation(name) else {
                bail!("{RULE_ID} option viewBounds: {name} does not resolve to one catalog view");
            };
            if resolved.insert(view.name.clone(), *mode).is_some() {
                bail!("{RULE_ID} option viewBounds: {name} resolves to a duplicate catalog view");
            }
        }
        Ok(Self(resolved))
    }

    pub(super) fn get<'a>(
        &self,
        name: &str,
        catalog: &'a SchemaCatalog,
    ) -> Option<(&'a CatalogView, ViewBoundMode)> {
        if self.0.is_empty() {
            return None;
        }
        let view = catalog.bounded_view_relation(name)?;
        let mode = self.0.get(&view.name)?;
        Some((view, *mode))
    }
}
