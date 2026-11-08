use crate::codebase::postgres::statements::SqlTransformedColumnUseFact;
mod anchored;
#[path = "view_columns.rs"]
mod view_columns;
use crate::codebase::postgres::{SchemaCatalog, SqlBranchJudgment};
use std::collections::BTreeMap;

pub(super) struct Columns<'a> {
    catalog: &'a SchemaCatalog,
    tables: BTreeMap<&'a str, BTreeMap<String, bool>>,
}
impl<'a> Columns<'a> {
    pub(super) fn new(catalog: &'a SchemaCatalog) -> Self {
        let mut tables: BTreeMap<_, _> = catalog
            .tables()
            .map(|table| {
                (
                    table.name.as_str(),
                    table
                        .columns
                        .iter()
                        .map(|column| (column.name.clone(), is_uuid(&column.data_type)))
                        .collect(),
                )
            })
            .collect();
        for view in catalog.views() {
            if let Some(columns) = view_columns::mapped(catalog, view, &tables) {
                tables.insert(view.name.as_str(), columns);
            }
        }
        Self { catalog, tables }
    }
    pub(super) fn resolve(
        &self,
        use_: &SqlTransformedColumnUseFact,
    ) -> Result<(&'a str, bool), &'static str> {
        let (table, uuid) =
            self.resolve_owner(&use_.table, &use_.column, use_.candidate_tables.as_deref())?;
        let drives_uuid_probe = use_.driving_probe.as_ref().is_some_and(|probe| {
            self.resolve_owner(
                &probe.table,
                &probe.column,
                probe.candidate_tables.as_deref(),
            )
            .is_ok_and(|(table, uuid)| {
                uuid && (probe.table.is_empty()
                    || self.catalog.same_relation(&probe.table, table)
                    || self
                        .catalog
                        .view_relation(&probe.table)
                        .is_some_and(|view| view.name == table))
            })
        });
        Ok((table, uuid && !drives_uuid_probe && !self.anchored(use_)))
    }
    fn resolve_owner(
        &self,
        table: &str,
        column: &str,
        candidates: Option<&[String]>,
    ) -> Result<(&'a str, bool), &'static str> {
        if !table.is_empty() {
            return self.column(table, column).ok_or("unresolved-column-owner");
        }
        let candidates = candidates.ok_or("unresolved-column-owner")?;
        let mut owner = None;
        for candidate in candidates {
            let table = self.name(candidate).ok_or("unresolved-column-owner")?;
            let value = self
                .tables
                .get(table)
                .and_then(|columns| columns.get(column));
            if value.is_none() && self.catalog.view_relation(candidate).is_some() {
                return Err("unresolved-column-owner");
            }
            if let Some(is_uuid) = value {
                if owner.is_some() {
                    return Err("ambiguous-column-owner");
                }
                owner = Some((table, *is_uuid));
            }
        }
        owner.ok_or("unresolved-column-owner")
    }
    fn name(&self, table: &str) -> Option<&'a str> {
        self.catalog
            .bounded_relation(table)
            .map(|relation| relation.name.as_str())
            .or_else(|| {
                self.catalog
                    .view_relation(table)
                    .map(|view| view.name.as_str())
            })
    }
    fn column(&self, table: &str, column: &str) -> Option<(&'a str, bool)> {
        let name = self.name(table)?;
        Some((name, *self.tables.get(name)?.get(column)?))
    }
    pub(super) fn judge(&self, uses: &[SqlTransformedColumnUseFact]) -> SqlBranchJudgment {
        let mut unknown = None;
        for use_ in uses {
            match self.resolve(use_) {
                Ok((_, true)) => return SqlBranchJudgment::violation(true),
                Err(reason) => unknown = Some(reason),
                _ => {}
            }
        }
        unknown.map_or_else(SqlBranchJudgment::clear, SqlBranchJudgment::abstain)
    }
}
// Type and key policy live here, independently of syntactic predicate extraction.
fn is_uuid(data_type: &str) -> bool {
    let parts = crate::codebase::postgres::decoded_parts(data_type);
    matches!(parts.as_slice(), [name] if name == "uuid")
        || matches!(parts.as_slice(), [schema, name] if schema == "pg_catalog" && name == "uuid")
}
