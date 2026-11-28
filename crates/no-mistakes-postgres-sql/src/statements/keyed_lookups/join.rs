mod factor;
mod using_clause;

use super::inspect_exprs;
use super::normalize::{extract_lookup_key, LookupKey};
use super::scope::{ScopeSnapshot, ScopeStack};
use super::table::lookup_table;
use super::LookupHistory;
use crate::idents::{ident_key, object_name_identity, relation_suffix_name};
use crate::statement_facts::SqlDuplicateKeyedLookupFact;
use factor::register_factor;
use sqlparser::ast::{
    Join, JoinConstraint, JoinOperator, ObjectName, Spanned, TableAlias, TableWithJoins,
};
use using_clause::{column_name, UsingColumns};

pub(super) fn register_table_with_joins(
    table: &TableWithJoins,
    scopes: &mut ScopeStack,
    seen: &mut LookupHistory,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    register_join_chain(table, scopes, seen, out);
}

fn register_join_chain(
    table: &TableWithJoins,
    scopes: &mut ScopeStack,
    seen: &mut LookupHistory,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) -> UsingColumns {
    let left_start = scopes.binding_count();
    let mut using = register_factor(&table.relation, scopes, seen, out);
    for join in &table.joins {
        inspect_join(join, left_start, scopes, &mut using, seen, out);
    }
    using
}

pub(super) fn register_table_factor(
    factor: &sqlparser::ast::TableFactor,
    scopes: &mut ScopeStack,
    seen: &mut LookupHistory,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    register_factor(factor, scopes, seen, out);
}

fn inspect_join(
    join: &Join,
    left_start: usize,
    scopes: &mut ScopeStack,
    using: &mut UsingColumns,
    seen: &mut LookupHistory,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    let constraint = join_constraint(&join.join_operator);
    let target =
        lookup_table(&join.relation).map(|(name, alias)| (object_name_identity(name), alias));
    if let Some((relation, alias)) = &target {
        if let Some((key, display)) =
            join_lookup(relation, *alias, constraint, &scopes.snapshot(), using)
        {
            let start = join.relation.span().start;
            let line = start.line.max(1) as usize;
            let column = start.column.max(1) as usize;
            record_lookup(key, display, line, column, seen, out);
        }
    }
    let right_start = scopes.binding_count();
    register_factor(&join.relation, scopes, seen, out);
    using.advance(
        &join.join_operator,
        constraint,
        target.as_ref().map(|(relation, _)| relation.as_str()),
    );
    if let Some(JoinConstraint::On(expr)) = constraint {
        inspect_exprs(std::iter::once(expr), scopes, seen, out);
    }
    let end = scopes.binding_count();
    match &join.join_operator {
        JoinOperator::Left(_) | JoinOperator::LeftOuter(_) => {
            scopes.refresh_bindings(right_start..end);
        }
        JoinOperator::Right(_) | JoinOperator::RightOuter(_) => {
            scopes.refresh_bindings(left_start..right_start);
            using.invalidate_left();
        }
        JoinOperator::FullOuter(_) => {
            scopes.refresh_bindings(left_start..end);
            using.invalidate_left();
        }
        _ => {}
    }
    if let Some(JoinConstraint::UsingAlias(columns, alias)) = constraint {
        scopes.add_using_alias(
            ident_key(alias),
            columns
                .iter()
                .filter_map(column_name)
                .map(|name| {
                    let source = using.resolve(&name);
                    (name, source)
                })
                .collect(),
        );
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
    relation: &str,
    alias: Option<&TableAlias>,
    constraint: Option<&JoinConstraint>,
    outer: &ScopeSnapshot,
    using: &UsingColumns,
) -> Option<(LookupKey, String)> {
    match constraint? {
        JoinConstraint::On(expr) => {
            let self_qualifier = match alias {
                Some(alias) => ident_key(&alias.name),
                None => relation_suffix_name(relation),
            };
            extract_lookup_key(relation, &self_qualifier, false, expr, outer, &[], None)
        }
        JoinConstraint::Using(columns) | JoinConstraint::UsingAlias(columns, _) => {
            using_lookup(relation.to_owned(), columns, using)
        }
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
    seen: &mut LookupHistory,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    if let Some((first_line, first_column)) = seen.observe(&key, (line, column)) {
        out.push(SqlDuplicateKeyedLookupFact {
            relation: key.relation,
            key_display,
            first_line,
            first_column,
            line,
            column,
        });
    }
}
