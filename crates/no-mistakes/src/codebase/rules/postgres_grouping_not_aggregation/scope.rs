pub(super) mod definitions;
use crate::codebase::postgres::{CatalogTable, SchemaCatalog};
use std::collections::BTreeSet;
use std::ops::Deref;
pub(super) struct CatalogScope<'a> {
    pub catalog: &'a SchemaCatalog,
    pub parameters: &'a [(u32, u32)],
    pub shadowed: &'a BTreeSet<String>,
}
impl CatalogScope<'_> {
    pub fn query_shadowed(&self, query: &sqlparser::ast::Query) -> BTreeSet<String> {
        let mut shadowed = self.shadowed.clone();
        if let Some(with) = &query.with {
            shadowed.extend(
                with.cte_tables
                    .iter()
                    .map(|cte| super::expressions::ident(&cte.alias.name)),
            );
        }
        shadowed
    }
    pub fn in_query<T>(
        &self,
        query: &sqlparser::ast::Query,
        inspect: impl FnOnce(&CatalogScope<'_>) -> T,
    ) -> T {
        let shadowed = self.query_shadowed(query);
        inspect(&CatalogScope {
            catalog: self.catalog,
            parameters: self.parameters,
            shadowed: &shadowed,
        })
    }
    pub fn parameter(&self, expr: &sqlparser::ast::Expr) -> bool {
        parameter(expr, self.parameters)
    }

    pub fn pinned(&self, expr: &sqlparser::ast::Expr) -> bool {
        self.parameter(expr) || super::expressions::pinned_value(expr)
    }
    pub fn relation(&self, name: &str) -> Option<&CatalogTable> {
        let parts = crate::codebase::postgres::decoded_parts(name);
        if parts.len() == 1 && self.shadowed.contains(&parts[0]) {
            return None;
        }
        self.catalog.relation(name)
    }
}
impl Deref for CatalogScope<'_> {
    type Target = SchemaCatalog;
    fn deref(&self) -> &SchemaCatalog {
        self.catalog
    }
}

pub(super) fn parameter(expr: &sqlparser::ast::Expr, parameters: &[(u32, u32)]) -> bool {
    use sqlparser::ast::Expr;
    match super::expressions::bare(expr) {
        Expr::Identifier(id) => {
            id.quote_style.is_none()
                && parameters.contains(&(id.span.start.line as u32, id.span.start.column as u32))
        }
        Expr::Cast { expr, .. } => parameter(expr, parameters),
        _ => false,
    }
}
