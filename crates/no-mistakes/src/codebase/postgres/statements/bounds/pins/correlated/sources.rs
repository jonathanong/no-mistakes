use super::columns::{function_columns, projection_columns};
use super::sql_name;
use super::Scan;
use crate::codebase::postgres::idents::{ident_key, object_name_ident, object_name_key};
use sqlparser::ast::TableFactor;

impl Scan {
    pub(super) fn add_factor(&mut self, factor: &TableFactor) {
        let cte = match factor {
            TableFactor::Table { name, .. } if self.is_cte(name) => object_name_ident(name)
                .and_then(|name| self.ctes.get(&ident_key(name)))
                .cloned(),
            _ => None,
        };
        let Some(frame) = self.stack.last_mut() else {
            return;
        };
        match factor {
            TableFactor::Table {
                name, alias, args, ..
            } => {
                let own = alias
                    .as_ref()
                    .map(|alias| ident_key(&alias.name))
                    .or_else(|| object_name_ident(name).map(ident_key));
                frame.relations.extend(own);
                if alias.is_none() {
                    frame.relations.insert(object_name_key(name));
                }
                let columns =
                    if let Some(alias) = alias.as_ref().filter(|alias| !alias.columns.is_empty()) {
                        Some(
                            alias
                                .columns
                                .iter()
                                .map(|column| ident_key(&column.name))
                                .collect(),
                        )
                    } else if let Some(columns) = cte {
                        columns
                    } else if args.is_some() {
                        function_columns(name, alias)
                    } else {
                        frame.tables.push(sql_name(name));
                        return;
                    };
                match columns {
                    Some(columns) => frame.columns.extend(columns),
                    None => frame.foreign = true,
                }
            }
            TableFactor::Derived {
                subquery, alias, ..
            } => {
                if let Some(alias) = alias {
                    frame.relations.insert(ident_key(&alias.name));
                }
                let columns = match alias.as_ref().filter(|alias| !alias.columns.is_empty()) {
                    Some(alias) => Some(
                        alias
                            .columns
                            .iter()
                            .map(|column| ident_key(&column.name))
                            .collect(),
                    ),
                    None => projection_columns(subquery),
                };
                match columns {
                    Some(columns) => frame.columns.extend(columns),
                    None => frame.foreign = true,
                }
            }
            TableFactor::UNNEST {
                alias,
                array_exprs,
                with_ordinality,
                ..
            } => {
                let own = alias.as_ref().map(|alias| ident_key(&alias.name));
                frame.relations.extend(own.clone());
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
                let mut exposed: Vec<_> = (0..array_exprs.len())
                    .map(|_| {
                        if array_exprs.len() == 1 {
                            own.clone().unwrap_or_else(|| "unnest".to_string())
                        } else {
                            "unnest".to_string()
                        }
                    })
                    .collect();
                if *with_ordinality {
                    exposed.push("ordinality".to_string());
                }
                for (column, name) in exposed.iter_mut().zip(columns) {
                    *column = name;
                }
                frame.columns.extend(exposed);
            }
            // The factors inside a parenthesized join are visited on their own.
            TableFactor::NestedJoin { .. } => {}
            _ => frame.foreign = true,
        }
    }
}
