//! Non-recursive CTE definitions read prior bindings, not their own output names.
use super::{projection_columns, Scan};
use crate::codebase::postgres::idents::ident_key;
use sqlparser::ast::Query;

impl Scan {
    pub(super) fn prepare_ctes(&mut self, query: &Query) {
        if let Some(with) = &query.with {
            for cte in &with.cte_tables {
                let columns = if cte.alias.columns.is_empty() {
                    projection_columns(&cte.query)
                } else {
                    Some(
                        cte.alias
                            .columns
                            .iter()
                            .map(|column| ident_key(&column.name))
                            .collect(),
                    )
                };
                let name = ident_key(&cte.alias.name);
                if with.recursive {
                    self.ctes.insert(name, columns);
                } else {
                    // Identity is local to this borrowed AST traversal; it never affects output order.
                    self.pending_ctes
                        .insert(cte.query.as_ref() as *const Query as usize, (name, columns));
                }
            }
        }
    }

    pub(super) fn complete_cte(&mut self, query: &Query) {
        if let Some((name, columns)) = self.pending_ctes.remove(&(query as *const Query as usize)) {
            self.ctes.insert(name, columns);
        }
    }
}
