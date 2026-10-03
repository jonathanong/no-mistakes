use super::super::scalar_arrays::scalar_array;
use super::super::Frame;
use crate::codebase::postgres::idents::ident_key;
use sqlparser::ast::{Expr, TableAlias};

impl Frame {
    /// Resolve the visible names of a single UNNEST source in this query frame.
    pub(super) fn add_unnest(
        &mut self,
        alias: &Option<TableAlias>,
        array_exprs: &[Expr],
        with_ordinality: bool,
    ) {
        let own = alias.as_ref().map(|alias| ident_key(&alias.name));
        if let Some(own) = &own {
            self.whole_rows.insert(own.clone());
            self.relations.insert(vec![own.clone()]);
        }
        let columns: Vec<_> = alias
            .as_ref()
            .map(|alias| {
                alias
                    .columns
                    .iter()
                    .map(|column| ident_key(&column.name))
                    .collect()
            })
            .unwrap_or_default();
        if !array_exprs.iter().all(scalar_array) {
            // Composite attributes and undeclared suffix columns remain unknown.
            self.columns.extend(columns);
            self.foreign = true;
            return;
        }
        let mut exposed: Vec<_> = (0..array_exprs.len())
            .map(|_| {
                if array_exprs.len() == 1 {
                    own.clone().unwrap_or_else(|| "unnest".to_string())
                } else {
                    "unnest".to_string()
                }
            })
            .collect();
        if with_ordinality {
            exposed.push("ordinality".to_string());
        }
        for (column, name) in exposed.iter_mut().zip(columns) {
            *column = name;
        }
        self.columns.extend(exposed);
    }
}
