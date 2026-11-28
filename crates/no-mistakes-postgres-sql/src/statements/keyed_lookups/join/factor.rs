use super::super::normalize::LookupKey;
use super::super::scope::ScopeStack;
use super::super::table::lookup_table;
use super::super::{inspect_exprs, walk_query};
use super::register_join_chain;
use super::using_clause::UsingColumns;
use crate::idents::{function_arg_expr, ident_key, object_name_identity};
use crate::statement_facts::SqlDuplicateKeyedLookupFact;
use sqlparser::ast::TableFactor;
use std::collections::BTreeMap;

pub(super) fn register_factor(
    factor: &TableFactor,
    scopes: &mut ScopeStack,
    seen: &mut BTreeMap<LookupKey, (usize, usize)>,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) -> UsingColumns {
    let binding = match factor {
        TableFactor::Table {
            name, alias, args, ..
        } => {
            if let Some(args) = args {
                inspect_exprs(
                    args.args.iter().filter_map(function_arg_expr),
                    scopes,
                    seen,
                    out,
                );
            }
            Some(scopes.add_table(name, alias.as_ref()))
        }
        TableFactor::Function {
            name, alias, args, ..
        } => {
            inspect_exprs(args.iter().filter_map(function_arg_expr), scopes, seen, out);
            Some(scopes.add_table(name, alias.as_ref()))
        }
        TableFactor::UNNEST {
            alias, array_exprs, ..
        } => {
            inspect_exprs(array_exprs, scopes, seen, out);
            Some(register_table_function_alias(
                scopes,
                alias.as_ref(),
                "unnest",
            ))
        }
        TableFactor::JsonTable {
            json_expr, alias, ..
        } => {
            inspect_exprs(std::iter::once(json_expr), scopes, seen, out);
            Some(register_table_function_alias(
                scopes,
                alias.as_ref(),
                "json_table",
            ))
        }
        TableFactor::XmlTable {
            namespaces,
            row_expression,
            passing,
            alias,
            ..
        } => {
            let inputs = namespaces
                .iter()
                .map(|namespace| &namespace.uri)
                .chain(std::iter::once(row_expression))
                .chain(passing.arguments.iter().map(|argument| &argument.expr));
            inspect_exprs(inputs, scopes, seen, out);
            Some(register_table_function_alias(
                scopes,
                alias.as_ref(),
                "xmltable",
            ))
        }
        TableFactor::Derived {
            subquery, alias, ..
        } => {
            walk_query(subquery, scopes, out);
            scopes.add_alias_only(alias.as_ref())
        }
        TableFactor::NestedJoin {
            table_with_joins,
            alias,
            ..
        } => {
            if alias.is_none() {
                return register_join_chain(table_with_joins, scopes, seen, out);
            }
            scopes.push_frame();
            register_join_chain(table_with_joins, scopes, &mut BTreeMap::new(), out);
            scopes.pop_frame();
            scopes.add_alias_only(alias.as_ref())
        }
        _ => None,
    };
    UsingColumns::for_binding(
        binding,
        lookup_table(factor).map(|(name, _)| object_name_identity(name)),
    )
}

fn register_table_function_alias(
    scopes: &mut ScopeStack,
    alias: Option<&sqlparser::ast::TableAlias>,
    default: &str,
) -> usize {
    let qualifier = alias.map_or_else(|| default.to_owned(), |alias| ident_key(&alias.name));
    scopes.add_named_alias(qualifier)
}
