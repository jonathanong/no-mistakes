use super::normalize::{extract_lookup_key, LookupKey};
use super::scope::{ScopeSnapshot, ScopeStack};
use super::walk_query;
use crate::codebase::postgres::idents::{ident_key, object_name_identity, relation_suffix_name};
use crate::codebase::postgres::statement_facts::SqlDuplicateKeyedLookupFact;
use sqlparser::ast::{
    Join, JoinConstraint, JoinOperator, ObjectName, Spanned, TableAlias, TableFactor,
    TableWithJoins,
};
use std::collections::BTreeMap;

pub(super) fn register_table_with_joins(
    table: &TableWithJoins,
    scopes: &mut ScopeStack,
    seen: &mut BTreeMap<LookupKey, (usize, usize)>,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    register_table_factor(&table.relation, scopes, out);
    for join in &table.joins {
        inspect_join(join, scopes, seen, out);
    }
}

pub(super) fn register_table_factor(
    factor: &TableFactor,
    scopes: &mut ScopeStack,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    match factor {
        TableFactor::Table {
            name,
            alias,
            args: None,
            ..
        } => {
            scopes.add_table(name, alias.as_ref());
        }
        TableFactor::Derived {
            subquery, alias, ..
        } => {
            walk_query(subquery, scopes, out);
            scopes.add_alias_only(alias.as_ref());
        }
        TableFactor::NestedJoin {
            table_with_joins, ..
        } => {
            let mut local_seen = BTreeMap::new();
            register_table_with_joins(table_with_joins, scopes, &mut local_seen, out);
        }
        TableFactor::Table { alias, .. } | TableFactor::Function { alias, .. } => {
            scopes.add_alias_only(alias.as_ref());
        }
        _ => {}
    }
}

pub(super) fn inspect_join(
    join: &Join,
    scopes: &mut ScopeStack,
    seen: &mut BTreeMap<LookupKey, (usize, usize)>,
    out: &mut Vec<SqlDuplicateKeyedLookupFact>,
) {
    let before = scopes.snapshot();
    let candidate = match &join.relation {
        TableFactor::Table {
            name,
            alias,
            args: None,
            ..
        } => join_lookup(join, name, alias.as_ref(), &before),
        _ => None,
    };
    if let Some((key, key_display, line, column)) = candidate {
        record_lookup(key, key_display, line, column, false, seen, out);
    }
    register_table_factor(&join.relation, scopes, out);
}

fn join_lookup(
    join: &Join,
    name: &ObjectName,
    alias: Option<&TableAlias>,
    outer: &ScopeSnapshot,
) -> Option<(LookupKey, String, usize, usize)> {
    let constraint = match &join.join_operator {
        JoinOperator::Join(c)
        | JoinOperator::Inner(c)
        | JoinOperator::Left(c)
        | JoinOperator::LeftOuter(c)
        | JoinOperator::Right(c)
        | JoinOperator::RightOuter(c)
        | JoinOperator::FullOuter(c) => c,
        _ => return None,
    };
    let relation = object_name_identity(name);
    let self_qualifier = match alias {
        Some(alias) => ident_key(&alias.name),
        None => relation_suffix_name(&relation),
    };
    let start = join.relation.span().start;
    let line = start.line.max(1) as usize;
    let column = start.column.max(1) as usize;
    match constraint {
        JoinConstraint::On(expr) => {
            let (key, display) =
                extract_lookup_key(&relation, &self_qualifier, false, expr, outer, &[])?;
            Some((key, display, line, column))
        }
        JoinConstraint::Using(cols) => {
            let binding = outer.single_current_binding()?;
            if cols.is_empty() {
                return None;
            }
            let mut equalities = Vec::new();
            let mut display = Vec::new();
            for col in cols {
                let name = col
                    .0
                    .last()
                    .and_then(|part| part.as_ident())
                    .map(ident_key)?;
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
                },
                format!("USING ({})", display.join(", ")),
                line,
                column,
            ))
        }
        _ => None,
    }
}

pub(super) fn record_lookup(
    key: LookupKey,
    key_display: String,
    line: usize,
    column: usize,
    conditional: bool,
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
    } else if !conditional {
        seen.insert(key, (line, column));
    }
}
