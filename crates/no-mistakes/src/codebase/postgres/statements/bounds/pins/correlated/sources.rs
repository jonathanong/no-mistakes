mod unnest;

use super::columns::{function_columns, partial_alias_columns, projection_columns};
use super::scalar_arrays::scalar_array;
use super::sql_name;
use super::Scan;
use crate::codebase::postgres::idents::{ident_key, object_name_ident};
use sqlparser::ast::{FunctionArg, FunctionArgExpr, TableFactor};

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
                name,
                alias,
                args,
                with_ordinality,
                ..
            } => {
                let own = alias
                    .as_ref()
                    .map(|alias| ident_key(&alias.name))
                    .or_else(|| object_name_ident(name).map(ident_key));
                if let Some(own) = own {
                    frame.scope.whole_rows.insert(own.clone());
                    frame.scope.relations.insert(vec![own]);
                }
                if alias.is_none() && args.is_none() {
                    frame.scope.relations.insert(
                        name.0
                            .iter()
                            .filter_map(|part| part.as_ident())
                            .map(ident_key)
                            .collect(),
                    );
                }
                let mut columns = if let Some(alias) =
                    alias.as_ref().filter(|alias| !alias.columns.is_empty())
                {
                    Some(
                        alias
                            .columns
                            .iter()
                            .map(|column| ident_key(&column.name))
                            .collect(),
                    )
                } else if let Some(columns) = cte {
                    columns
                } else if let Some(args) = args {
                    if object_name_ident(name).is_some_and(|name| ident_key(name) == "unnest")
                            && !args.args.iter().all(|argument| matches!(argument, FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) if scalar_array(expr)))
                        { None } else { function_columns(name, alias) }
                } else {
                    frame.scope.tables.push(sql_name(name));
                    return;
                };
                if args.is_some()
                    && *with_ordinality
                    && function_columns(name, &None).is_some()
                    && alias.as_ref().map_or(0, |alias| alias.columns.len()) < 2
                {
                    if let Some(columns) = &mut columns {
                        columns.insert("ordinality".to_string());
                    }
                }
                match columns {
                    Some(columns) => frame.scope.columns.extend(columns),
                    None => frame.scope.foreign = true,
                }
            }
            TableFactor::Derived {
                subquery, alias, ..
            } => {
                if let Some(alias) = alias {
                    let name = ident_key(&alias.name);
                    frame.scope.whole_rows.insert(name.clone());
                    frame.scope.relations.insert(vec![name]);
                }
                let columns = match alias.as_ref().filter(|alias| !alias.columns.is_empty()) {
                    Some(alias) => partial_alias_columns(subquery, alias),
                    None => projection_columns(subquery),
                };
                match columns {
                    Some(columns) => frame.scope.columns.extend(columns),
                    None => frame.scope.foreign = true,
                }
            }
            TableFactor::UNNEST {
                alias,
                array_exprs,
                with_ordinality,
                ..
            } => {
                frame.add_unnest(alias, array_exprs, *with_ordinality);
            }
            // The factors inside a parenthesized join are visited on their own.
            TableFactor::NestedJoin {
                alias: Some(alias), ..
            } => {
                let name = ident_key(&alias.name);
                frame.scope.whole_rows.insert(name.clone());
                frame.scope.relations.insert(vec![name]);
                // A relation alias alone preserves child output names. Only explicit
                // column renaming makes their catalog ownership unavailable here.
                frame.scope.foreign |= !alias.columns.is_empty();
            }
            TableFactor::NestedJoin { alias: None, .. } => {}
            _ => frame.scope.foreign = true,
        }
    }
}
