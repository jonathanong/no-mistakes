use super::normalize::{extract_lookup_key, LookupKey};
use super::scope::{ScopeSnapshot, ScopeStack};
use super::{inspect_exprs, walk_query};
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
    let mut using_provenance = Vec::new();
    if let Some((key, key_display, line, column, provenance)) = candidate {
        using_provenance = provenance;
        record_lookup(key, key_display, line, column, seen, out);
    }
    register_table_factor(&join.relation, scopes, out);
    let preserves_left_using = matches!(
        join.join_operator,
        JoinOperator::Join(_)
            | JoinOperator::Inner(_)
            | JoinOperator::Left(_)
            | JoinOperator::LeftOuter(_)
    );
    for (col, binding) in using_provenance {
        if preserves_left_using {
            scopes.record_using_column(col, binding);
        } else {
            scopes.invalidate_using_column(&col);
        }
    }
    if let Some(JoinConstraint::On(expr)) = join_constraint(&join.join_operator) {
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

type JoinLookupResult = (LookupKey, String, usize, usize, Vec<(String, usize)>);

fn join_lookup(
    join: &Join,
    name: &ObjectName,
    alias: Option<&TableAlias>,
    outer: &ScopeSnapshot,
) -> Option<JoinLookupResult> {
    if alias.is_some_and(|a| !a.columns.is_empty()) {
        return None;
    }
    let constraint = join_constraint(&join.join_operator)?;
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
                extract_lookup_key(&relation, &self_qualifier, false, expr, outer, &[], None)?;
            Some((key, display, line, column, Vec::new()))
        }
        JoinConstraint::Using(cols) => {
            if cols.is_empty() {
                return None;
            }
            let mut equalities = Vec::new();
            let mut display = Vec::new();
            let mut provenance = Vec::new();
            for col in cols {
                let name = col
                    .0
                    .last()
                    .and_then(|part| part.as_ident())
                    .map(ident_key)?;
                let binding = outer.resolve_using_column(&name)?;
                equalities.push((name.clone(), format!("b{binding}.{name}")));
                provenance.push((name.clone(), binding));
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
                line,
                column,
                provenance,
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
