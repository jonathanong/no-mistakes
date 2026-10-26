use crate::codebase::postgres::statements::SqlTransformedColumnUseFact;
use crate::codebase::postgres::{CatalogTable, SchemaCatalog, SqlBranchJudgment};
use std::collections::BTreeMap;

pub(super) struct Columns<'a> {
    catalog: &'a SchemaCatalog,
    tables: BTreeMap<&'a str, BTreeMap<&'a str, bool>>,
}
impl<'a> Columns<'a> {
    pub(super) fn new(catalog: &'a SchemaCatalog) -> Self {
        Self {
            catalog,
            tables: catalog
                .tables()
                .map(|table| {
                    (
                        table.name.as_str(),
                        table
                            .columns
                            .iter()
                            .map(|column| (column.name.as_str(), is_uuid(&column.data_type)))
                            .collect(),
                    )
                })
                .collect(),
        }
    }
    pub(super) fn resolve(
        &self,
        use_: &SqlTransformedColumnUseFact,
    ) -> Result<(&'a str, bool), &'static str> {
        if !use_.table.is_empty() {
            return self
                .column(&use_.table, &use_.column)
                .ok_or("unresolved-column-owner");
        }
        let candidates = use_
            .candidate_tables
            .as_ref()
            .ok_or("unresolved-column-owner")?;
        let mut owner = None;
        for candidate in candidates {
            let table = self
                .catalog
                .relation(candidate)
                .ok_or("unresolved-column-owner")?;
            if let Some(is_uuid) = self.tables[table.name.as_str()].get(use_.column.as_str()) {
                if owner.is_some() {
                    return Err("ambiguous-column-owner");
                }
                owner = Some((table.name.as_str(), *is_uuid));
            }
        }
        owner.ok_or("unresolved-column-owner")
    }
    fn column(&self, table: &str, column: &str) -> Option<(&'a str, bool)> {
        let relation: &'a CatalogTable = self.catalog.relation(table)?;
        Some((
            relation.name.as_str(),
            *self.tables.get(relation.name.as_str())?.get(column)?,
        ))
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
