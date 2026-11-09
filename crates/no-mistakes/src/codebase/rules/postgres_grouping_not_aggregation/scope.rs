use crate::codebase::postgres::{CatalogTable, SchemaCatalog};
use std::collections::BTreeSet;
use std::ops::Deref;
pub(super) struct CatalogScope<'a> {
    pub catalog: &'a SchemaCatalog,
    pub parameters: &'a [(u32, u32)],
    pub shadowed: &'a BTreeSet<String>,
}
impl CatalogScope<'_> {
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
