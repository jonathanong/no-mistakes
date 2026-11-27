use super::super::{prepared::PreparedPostgresFacts, SchemaCatalog};
use crate::codebase::check_facts::{CheckFactMap, CheckFactPlan};
use crate::codebase::postgres_analysis::MapPayload;
use crate::codebase::ts_source::SourceStore;
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, LazyLock},
};

mod access;
mod fragments;
mod postgres_catalog;
mod postgres_readable;
mod sql_locations;
mod sql_verdicts;
#[cfg(test)]
mod test_accessors;
pub use sql_verdicts::{SqlApplicationFacts, SqlFactContext};

#[derive(Clone, Default)]
pub(crate) struct PostgresMapFacts {
    pub(crate) prepared: PreparedPostgresFacts,
    pub(crate) catalogs: BTreeMap<String, Result<Arc<SchemaCatalog>, Arc<str>>>,
}

static EMPTY: LazyLock<PostgresMapFacts> = LazyLock::new(PostgresMapFacts::default);

impl CheckFactMap {
    pub(crate) fn postgres(&self) -> &PreparedPostgresFacts {
        &self.postgres_map().prepared
    }

    fn postgres_map(&self) -> &PostgresMapFacts {
        self.postgres_payload.as_ref().map_or_else(
            || &*EMPTY,
            |facts| facts.downcast_ref().expect("Postgres request fact payload"),
        )
    }
}

impl PostgresMapFacts {
    pub(crate) fn into_payload(self) -> MapPayload {
        Some(Arc::new(self))
    }
}

pub(crate) fn prepare(
    root: &Path,
    sources: &SourceStore,
    plan: &CheckFactPlan,
    map: &CheckFactMap,
) -> MapPayload {
    let catalogs =
        super::super::load_schema_catalogs(root, sources, &plan.postgres_schema_catalog_paths);
    let prepared = super::super::prepared::prepare(root, &map.files, sources, plan, map);
    PostgresMapFacts { prepared, catalogs }.into_payload()
}
