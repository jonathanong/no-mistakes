//! `statement.span()` and `Query::span()` both end at sqlparser's partial span.
//! `DO NOTHING` and `EXISTS` omit their closing syntax. The CTE closing `)` bounds
//! the nested statement. Parentheses opened outside that statement, as in
//! `AS ((INSERT ...))`, belong to the wrapper and stay outside the slice.

use super::super::locations::Locations;
use super::super::types::{PostgresSqlQuery, PostgresSqlQueryClause, PostgresSqlQueryUnsupported};
use sqlparser::ast::helpers::attached_token::AttachedToken;
use sqlparser::tokenizer::Token;

mod tail;
use tail::{bounded_end, open_depth};

const BOUNDARY: &str = "nested data-modifying statement boundary";

/// `true` when `scope` owns a nested statement. An index outside `facts.scopes`
/// is not a boundary, and a CTE with no matching `cte_id` does not need one.
pub(super) fn needs(facts: &PostgresSqlQuery, scope: usize) -> bool {
    let Some(definition) = facts.scopes.get(scope).map(|root| root.cte_definition_id) else {
        return false;
    };
    facts
        .nested_statements
        .iter()
        .any(|statement| statement.cte_id == definition)
}

pub(super) fn repair(
    facts: &mut PostgresSqlQuery,
    locations: &Locations<'_>,
    scope: usize,
    closing: &AttachedToken,
) {
    if !needs(facts, scope) {
        return;
    }
    let scopes = statement_scopes(facts, scope);
    let indexes = facts
        .nested_statements
        .iter()
        .enumerate()
        .filter(|(_, statement)| scopes.contains(&statement.query_scope_id))
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    for index in indexes {
        repair_statement(facts, locations, closing, index);
    }
}

/// The CTE scope plus `SetExpr::Query` wrappers around its body.
/// Set operations and nested CTEs are separate statements.
fn statement_scopes(facts: &PostgresSqlQuery, scope: usize) -> Vec<usize> {
    let Some(root) = facts.scopes.get(scope) else {
        return vec![scope];
    };
    let definition = root.cte_definition_id;
    let mut children = vec![Vec::new(); facts.scopes.len()];
    for child in &facts.scopes {
        if let Some(parent) = child.parent_scope_id {
            if let Some(bucket) = children.get_mut(parent) {
                bucket.push(child.id);
            }
        }
    }
    let mut accepted = Vec::new();
    let mut pending = vec![scope];
    let mut seen = vec![false; facts.scopes.len()];
    seen[scope] = true;
    while let Some(parent) = pending.pop() {
        accepted.push(parent);
        if facts.scopes[parent].set_operation.is_some() {
            continue;
        }
        for child in children[parent].iter().copied() {
            // Scope ids are vec indexes. An id outside the vec is not a child.
            if child >= seen.len() || seen[child] {
                continue;
            }
            let child_scope = &facts.scopes[child];
            if child_scope.clause != PostgresSqlQueryClause::SetBranch
                || child_scope.cte_definition_id != definition
            {
                continue;
            }
            seen[child] = true;
            pending.push(child);
        }
    }
    accepted
}

fn repair_statement(
    facts: &mut PostgresSqlQuery,
    locations: &Locations<'_>,
    closing: &AttachedToken,
    index: usize,
) {
    let Some(current) = facts.nested_statements[index].span.clone() else {
        return;
    };
    let scope = facts.nested_statements[index].query_scope_id;
    let Some(boundary) = closing_offset(closing, locations) else {
        reject(facts, index, scope);
        return;
    };
    if current.start.offset > boundary || current.end.offset > boundary {
        reject(facts, index, scope);
        return;
    }
    let depth = match open_depth(locations.slice(&current)) {
        Ok(depth) => depth,
        Err(()) => {
            reject(facts, index, scope);
            return;
        }
    };
    let tail = locations.slice(&locations.range(current.end.offset, boundary));
    let extra = match bounded_end(tail, depth) {
        Ok(end) => end.unwrap_or(0),
        Err(()) => {
            reject(facts, index, scope);
            return;
        }
    };
    if extra == 0 {
        return;
    }
    let mut span = current;
    let end = span.end.offset + extra;
    span.end = locations.range(end, end).start;
    let sql = locations.slice(&span).to_owned();
    let statement = &mut facts.nested_statements[index];
    statement.span = Some(span);
    statement.sql = sql;
}

fn closing_offset(closing: &AttachedToken, locations: &Locations<'_>) -> Option<usize> {
    if closing.0.token != Token::RParen {
        return None;
    }
    Some(locations.position(closing.0.span.start)?.offset)
}

fn reject(facts: &mut PostgresSqlQuery, index: usize, scope: usize) {
    let statement = &mut facts.nested_statements[index];
    let partial = statement.span.clone();
    statement.sql.clear();
    statement.span = None;
    statement.complete = false;
    let item = PostgresSqlQueryUnsupported {
        scope_id: scope,
        clause: PostgresSqlQueryClause::Other,
        reason: BOUNDARY.into(),
        span: partial,
    };
    statement.unsupported.push(item.clone());
    facts.unsupported.push(item);
    facts.complete = false;
}

#[cfg(test)]
mod tests;
