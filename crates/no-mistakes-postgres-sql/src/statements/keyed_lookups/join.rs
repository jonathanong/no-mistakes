mod using_clause;

use super::normalize::{extract_lookup_key, LookupKey};
use super::scope::{ScopeSnapshot, ScopeStack};
use super::table::lookup_table;
use super::{inspect_exprs, walk_query};
use crate::idents::{ident_key, object_name_identity, relation_suffix_name};
use crate::statement_facts::SqlDuplicateKeyedLookupFact;
use sqlparser::ast::{
    Join, JoinConstraint, JoinOperator, ObjectName, Spanned, TableAlias, TableFactor,
    TableWithJoins,
};
use std::collections::BTreeMap;
use using_clause::{column_name, UsingColumns};

pub(super) fn register_table_with_joins(
    table: &TableWithJoins,
    scopes: &mut ScopeStack,
    seen: &mut BTreeMap<LookupKey, (usize, usize)>,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    register_join_chain(table, scopes, seen, out);
}

fn register_join_chain(
    table: &TableWithJoins,
    scopes: &mut ScopeStack,
    seen: &mut BTreeMap<LookupKey, (usize, usize)>,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) -> UsingColumns {
    let mut using = register_factor(&table.relation, scopes, out);
    for join in &table.joins {
        inspect_join(join, scopes, &mut using, seen, out);
    }
    using
}

pub(super) fn register_table_factor(
    factor: &TableFactor,
    scopes: &mut ScopeStack,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    register_factor(factor, scopes, out);
}

fn register_factor(
    factor: &TableFactor,
    scopes: &mut ScopeStack,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) -> UsingColumns {
    let binding = match factor {
        TableFactor::Table {
            name,
            alias,
            args: None,
            ..
        } => Some(scopes.add_table(name, alias.as_ref())),
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
            let mut local_seen = BTreeMap::new();
            if alias.is_none() {
                return register_join_chain(table_with_joins, scopes, &mut local_seen, out);
            }
            scopes.push_frame();
            register_join_chain(table_with_joins, scopes, &mut local_seen, out);
            scopes.pop_frame();
            scopes.add_alias_only(alias.as_ref())
        }
        TableFactor::Table { alias, .. } | TableFactor::Function { alias, .. } => {
            scopes.add_alias_only(alias.as_ref())
        }
        _ => None,
    };
    UsingColumns::for_binding(binding)
}

fn inspect_join(
    join: &Join,
    scopes: &mut ScopeStack,
    using: &mut UsingColumns,
    seen: &mut BTreeMap<LookupKey, (usize, usize)>,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    let constraint = join_constraint(&join.join_operator);
    if let Some((name, alias)) = lookup_table(&join.relation) {
        if let Some((key, display)) =
            join_lookup(name, alias, constraint, &scopes.snapshot(), using)
        {
            let start = join.relation.span().start;
            let line = start.line.max(1) as usize;
            let column = start.column.max(1) as usize;
            record_lookup(key, display, line, column, seen, out);
        }
    }
    register_table_factor(&join.relation, scopes, out);
    using.advance(&join.join_operator, constraint);
    if let Some(JoinConstraint::On(expr)) = constraint {
        inspect_exprs(std::iter::once(expr), scopes, seen, out);
    }
}

fn join_constraint(op: &JoinOperator) -> Option<&JoinConstraint> {
    match op {
        JoinOperator::Join(c)
        | JoinOperator::Inner(c)
        | JoinOperator::Left(c)
        | JoinOperator::LeftOuter(c)
        | JoinOperator::Right(c)
        | JoinOperator::RightOuter(c)
        | JoinOperator::FullOuter(c) => Some(c),
        _ => None,
    }
}

fn join_lookup(
    name: &ObjectName,
    alias: Option<&TableAlias>,
    constraint: Option<&JoinConstraint>,
    outer: &ScopeSnapshot,
    using: &UsingColumns,
) -> Option<(LookupKey, String)> {
    let relation = object_name_identity(name);
    match constraint? {
        JoinConstraint::On(expr) => {
            let self_qualifier = match alias {
                Some(alias) => ident_key(&alias.name),
                None => relation_suffix_name(&relation),
            };
            extract_lookup_key(&relation, &self_qualifier, false, expr, outer, &[], None)
        }
        JoinConstraint::Using(columns) => using_lookup(relation, columns, using),
        _ => None,
    }
}

fn using_lookup(
    relation: String,
    columns: &[ObjectName],
    using: &UsingColumns,
) -> Option<(LookupKey, String)> {
    if columns.is_empty() {
        return None;
    }
    let mut equalities = Vec::new();
    let mut display = Vec::new();
    for column in columns {
        let name = column_name(column)?;
        let binding = using.resolve(&name)?;
        equalities.push((name.clone(), format!("b{binding}.{name}")));
        display.push(name);
    }
    equalities.sort();
    Some((
        LookupKey {
            relation,
            equalities,
            residuals: Vec::new(),
            order_by: Vec::new(),
            limit: None,
        },
        format!("USING ({})", display.join(", ")),
    ))
}

pub(super) fn record_lookup(
    key: LookupKey,
    key_display: String,
    line: usize,
    column: usize,
    seen: &mut BTreeMap<LookupKey, (usize, usize)>,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    if let Some(&(first_line, first_column)) = seen.get(&key) {
        out.push(SqlDuplicateKeyedLookupFact {
            relation: key.relation,
            key_display,
            first_line,
            first_column,
            line,
            column,
        });
    } else {
        seen.insert(key, (line, column));
    }
}
