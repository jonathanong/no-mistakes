use super::super::scope::ScopeStack;
use super::super::table::lookup_table;
use super::super::LookupHistory;
use super::super::{inspect_exprs, walk_query};
use super::register_join_chain;
use super::using_clause::UsingColumns;
use crate::idents::{function_arg_expr, ident_key, object_name_identity};
use crate::statement_facts::SqlDuplicateKeyedLookupFact;
use sqlparser::ast::{TableFactor, XmlTableColumnOption};

pub(super) fn register_factor(
    factor: &TableFactor,
    scopes: &mut ScopeStack,
    seen: &mut LookupHistory,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
    evaluate: bool,
) -> UsingColumns {
    if evaluate {
        inspect_factor_inputs(factor, scopes, seen, out);
    }
    let binding = match factor {
        TableFactor::Table { name, alias, .. } | TableFactor::Function { name, alias, .. } => {
            Some(scopes.add_table(name, alias.as_ref()))
        }
        TableFactor::UNNEST { alias, .. } => Some(register_table_function_alias(
            scopes,
            alias.as_ref(),
            "unnest",
        )),
        TableFactor::JsonTable { alias, .. } => Some(register_table_function_alias(
            scopes,
            alias.as_ref(),
            "json_table",
        )),
        TableFactor::XmlTable { alias, .. } => Some(register_table_function_alias(
            scopes,
            alias.as_ref(),
            "xmltable",
        )),
        TableFactor::Derived { alias, .. } => scopes.add_alias_only(alias.as_ref()),
        TableFactor::NestedJoin {
            table_with_joins,
            alias,
            ..
        } => {
            if alias.is_none() {
                return register_join_chain(table_with_joins, scopes, seen, out, evaluate);
            }
            let first = scopes.binding_count();
            register_join_chain(
                table_with_joins,
                scopes,
                &mut LookupHistory::new(),
                out,
                evaluate,
            );
            scopes.truncate_bindings(first);
            scopes.add_alias_only(alias.as_ref())
        }
        _ => None,
    };
    UsingColumns::for_binding(
        binding,
        lookup_table(factor).map(|(name, _)| object_name_identity(name)),
    )
}

fn inspect_factor_inputs(
    factor: &TableFactor,
    scopes: &mut ScopeStack,
    seen: &mut LookupHistory,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    match factor {
        TableFactor::Table {
            args: Some(args), ..
        } => {
            inspect_exprs(
                args.args.iter().filter_map(function_arg_expr),
                scopes,
                seen,
                out,
            );
        }
        TableFactor::Function { args, .. } => {
            inspect_exprs(args.iter().filter_map(function_arg_expr), scopes, seen, out);
        }
        TableFactor::UNNEST { array_exprs, .. } => inspect_exprs(array_exprs, scopes, seen, out),
        TableFactor::JsonTable { json_expr, .. } => {
            inspect_exprs(std::iter::once(json_expr), scopes, seen, out);
        }
        TableFactor::XmlTable {
            namespaces,
            row_expression,
            passing,
            columns,
            ..
        } => {
            let paths = columns.iter().filter_map(|column| match &column.option {
                XmlTableColumnOption::NamedInfo { path, .. } => path.as_ref(),
                XmlTableColumnOption::ForOrdinality => None,
            });
            let inputs = namespaces
                .iter()
                .map(|namespace| &namespace.uri)
                .chain(std::iter::once(row_expression))
                .chain(passing.arguments.iter().map(|argument| &argument.expr))
                .chain(paths);
            inspect_exprs(inputs, scopes, seen, out);
            for default in columns.iter().filter_map(|column| match &column.option {
                XmlTableColumnOption::NamedInfo { default, .. } => default.as_ref(),
                XmlTableColumnOption::ForOrdinality => None,
            }) {
                inspect_exprs(
                    std::iter::once(default),
                    scopes,
                    &mut LookupHistory::new(),
                    out,
                );
            }
        }
        TableFactor::Derived {
            lateral, subquery, ..
        } => {
            let first_visible = if *lateral { 0 } else { scopes.binding_count() };
            scopes.with_local_bindings(first_visible, |scopes| walk_query(subquery, scopes, out));
        }
        _ => {}
    }
}

fn register_table_function_alias(
    scopes: &mut ScopeStack,
    alias: Option<&sqlparser::ast::TableAlias>,
    default: &str,
) -> usize {
    let qualifier = alias.map_or_else(|| default.to_owned(), |alias| ident_key(&alias.name));
    scopes.add_named_alias(qualifier)
}
