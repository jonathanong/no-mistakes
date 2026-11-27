use super::super::normalize::LookupKey;
use super::super::scope::ScopeStack;
use super::super::{inspect_exprs, walk_query};
use super::register_join_chain;
use super::using_clause::UsingColumns;
use crate::idents::function_arg_expr;
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
    UsingColumns::for_binding(binding)
}
