//! Literal ternaries and logical clauses in an untagged template become SQL alternatives.
//! A reached unknown branch abstains for the whole template.
use super::super::super::resolve::functions::LocalFunctions;

// Unknown helpers abstain before argument classification walks their subtrees.
fn known_helper_args<T>(
    functions: &LocalFunctions,
    name: &str,
    classify: impl FnOnce() -> Option<T>,
) -> Option<T> {
    if !functions.has_program(name) {
        return None;
    }
    classify()
}

#[rustfmt::skip]
mod literal {
use super::super::super::super::ScopeVisitor;
use super::super::super::conditions::{choice_id, truth};
use super::super::super::limit::binary_alternatives;
use super::super::super::{combine, Recovered, ValueKind};
use super::super::{effects, piece};
use crate::codebase::postgres::embedded::{source_positions, EmbeddedSqlKind};
use crate::codebase::ts_source::{byte_offset_to_line, unwrap_ts_wrappers};
use oxc_ast::ast::{ConditionalExpression, Expression, TemplateLiteral};
use oxc_span::GetSpan;

mod logic;

pub(in crate::codebase::postgres::embedded::walk::variants) fn recover(visitor: &ScopeVisitor<'_>, template: &TemplateLiteral<'_>, depth: u8, constraints: &[(u64, u32)]) -> Option<Vec<Recovered>> {
    if depth == 0 || effects::mutates_captured_fragment(visitor, template) { return None; }
    let line = byte_offset_to_line(visitor.source, template.span.start as usize);
    let mut versions = vec![Recovered::empty(line)];
    for (index, quasi) in template.quasis.iter().enumerate() {
        versions = combine(versions, vec![piece::quasi(visitor, quasi)])?;
        if let Some(expr) = template.expressions.get(index) {
            let values = clause(visitor, expr, depth - 1, constraints)?;
            if !plain(&values) { return None; }
            versions = combine(versions, values)?;
        }
    }
    // NUL is a deferred const-splice marker; dropping only those arms is partial.
    if versions.iter().any(|value| value.sql.contains('\0')) { return None; }
    for version in &mut versions { version.enumerated = true; version.fragment = false; }
    Some(versions)
}

pub(in crate::codebase::postgres::embedded::walk::variants) fn observe_const(visitor: &mut ScopeVisitor<'_>, name: &str, init: Option<&Expression<'_>>, variants: Option<&[Recovered]>) -> Option<Vec<Recovered>> {
    let init = init?;
    if !visitor.declaring_const || !clause_init(init) || visitor.lookup(name).is_some_and(|binding| binding.builder_identity.is_some()) {
        return None;
    }
    let [constraints] = visitor.variant_paths.as_slice() else { return None };
    let recovered = clause(visitor, init, 8, constraints)?;
    let index = scope_index(visitor, name)?;
    if let Some(stored) = variants {
        // Keep a real walker snapshot. An absent-fragment artifact is not a clause union.
        if plain(stored) && same_sql(stored, &recovered) {
            visitor.clause_unions.insert((index, name.to_string()));
        }
        return None;
    }
    visitor.clause_unions.insert((index, name.to_string()));
    Some(recovered)
}

fn clause_init(init: &Expression<'_>) -> bool {
    matches!(unwrap_ts_wrappers(init), Expression::ConditionalExpression(_) | Expression::LogicalExpression(_))
}

fn clause(visitor: &ScopeVisitor<'_>, expr: &Expression<'_>, depth: u8, constraints: &[(u64, u32)]) -> Option<Vec<Recovered>> {
    if depth == 0 { return None; }
    let expr = unwrap_ts_wrappers(expr);
    let line = byte_offset_to_line(visitor.source, expr.span().start as usize);
    match expr {
        Expression::StringLiteral(literal) => text(visitor, expr, literal.value.to_string(), line),
        Expression::TemplateLiteral(template) => recover(visitor, template, depth - 1, constraints),
        Expression::Identifier(_) => constant(visitor, expr, constraints),
        Expression::ConditionalExpression(branch) => conditional(visitor, branch, depth, constraints),
        Expression::LogicalExpression(branch) => logic::recover(visitor, branch, depth, constraints),
        Expression::CallExpression(call) => call_clause(visitor, expr, call, line),
        _ => None,
    }
}

fn conditional(visitor: &ScopeVisitor<'_>, branch: &ConditionalExpression<'_>, depth: u8, constraints: &[(u64, u32)]) -> Option<Vec<Recovered>> {
    if visitor.expression_mutates_builder(&branch.test) { return None; }
    let (id, truth_arm) = choice_id(visitor, &branch.test, branch.span.start);
    let taken = constraints.iter().find(|(other, _)| *other == id).map(|(_, choice)| *choice == truth_arm).or_else(|| truth(&branch.test));
    if let Some(taken) = taken {
        return clause(visitor, if taken { &branch.consequent } else { &branch.alternate }, depth - 1, constraints);
    }
    let mut yes = constraints.to_vec();
    yes.push((id, truth_arm));
    let mut no = constraints.to_vec();
    no.push((id, 1 - truth_arm));
    let mut left = clause(visitor, &branch.consequent, depth - 1, &yes)?;
    let mut right = clause(visitor, &branch.alternate, depth - 1, &no)?;
    for value in &mut left { value.choices.push((id, truth_arm)); }
    for value in &mut right { value.choices.push((id, 1 - truth_arm)); }
    binary_alternatives(left, right, id)
}

fn constant(visitor: &ScopeVisitor<'_>, expr: &Expression<'_>, constraints: &[(u64, u32)]) -> Option<Vec<Recovered>> {
    let Expression::Identifier(ident) = expr else { return None };
    let name = ident.name.as_str();
    let binding = visitor.lookup(name)?;
    if binding.builder_identity.is_some() { return None; }
    let variants = binding.variants?;
    let values = if binding.kind == EmbeddedSqlKind::ImmutableLocal {
        let sql = binding.sql?;
        (variants.len() == 1 && variants[0].sql == sql).then(|| keep(variants, constraints)).flatten()?
    } else {
        is_union(visitor, name).then(|| keep(variants, constraints)).flatten()?
    };
    // One shared declaration must not collapse every caller into one finding.
    Some(at_reference(visitor, expr, values))
}

fn at_reference(visitor: &ScopeVisitor<'_>, expr: &Expression<'_>, mut values: Vec<Recovered>) -> Vec<Recovered> {
    let start = expr.span().start;
    let line = byte_offset_to_line(visitor.source, start as usize);
    let positions = source_positions::for_expression(expr, visitor.source, start as usize, line);
    for value in &mut values {
        value.line = line;
        value.origins = vec![start; value.sql.len()];
        value.positions = positions.clone();
    }
    values
}

fn text(visitor: &ScopeVisitor<'_>, expr: &Expression<'_>, sql: String, line: u32) -> Option<Vec<Recovered>> {
    if tainted(&sql) { return None; }
    let mut value = Recovered::empty(line);
    value.origins = source_positions::origins::expression(expr, visitor.source, &sql);
    value.positions = source_positions::for_expression(expr, visitor.source, expr.span().start as usize, line);
    if value.origins.len() != sql.len() { return None; }
    value.sql = sql;
    Some(vec![value])
}

fn keep(values: Vec<Recovered>, constraints: &[(u64, u32)]) -> Option<Vec<Recovered>> {
    let values: Vec<_> = values.into_iter().filter(|value| agrees(value, constraints)).collect();
    (!values.is_empty() && plain(&values)).then_some(values)
}
fn agrees(value: &Recovered, constraints: &[(u64, u32)]) -> bool {
    !value.choices.iter().any(|(id, arm)| constraints.iter().any(|(other, choice)| id == other && arm != choice))
}
fn plain(values: &[Recovered]) -> bool {
    values.iter().all(|value| value.value == ValueKind::Sql && !value.fragment && !tainted(&value.sql))
}
fn tainted(sql: &str) -> bool {
    sql.contains('\0') || crate::codebase::postgres::embedded::helpers::sentinel::contains(sql)
}
fn same_sql(left: &[Recovered], right: &[Recovered]) -> bool {
    let mut left: Vec<_> = left.iter().map(|value| value.sql.as_str()).collect();
    let mut right: Vec<_> = right.iter().map(|value| value.sql.as_str()).collect();
    left.sort_unstable();
    right.sort_unstable();
    left == right
}
fn is_union(visitor: &ScopeVisitor<'_>, name: &str) -> bool {
    scope_index(visitor, name).is_some_and(|index| visitor.clause_unions.contains(&(index, name.to_string())))
}
fn scope_index(visitor: &ScopeVisitor<'_>, name: &str) -> Option<usize> {
    visitor.scopes.iter().enumerate().rfind(|(_, scope)| scope.contains_key(name)).map(|(index, _)| index)
}

fn call_clause(visitor: &ScopeVisitor<'_>, expr: &Expression<'_>, call: &oxc_ast::ast::CallExpression<'_>, line: u32) -> Option<Vec<Recovered>> {
    let oxc_ast::ast::Expression::Identifier(name) = unwrap_ts_wrappers(&call.callee) else { return None };
    if call.optional || visitor.shadowed_locally(name.name.as_str()) { return None; }
    let args = super::known_helper_args(&visitor.functions, name.name.as_str(), || super::super::super::super::imported::clause_args(visitor, call))?;
    let strings = visitor.functions.identifier_clauses(name.name.as_str(), &args)?;
    let mut values = Vec::new();
    for sql in strings { values.extend(text(visitor, expr, sql, line)?); }
    super::super::super::limit::bounded(values)
}

}

pub(in crate::codebase::postgres::embedded::walk::variants) use literal::{observe_const, recover};

#[cfg(test)]
#[path = "clauses/tests.rs"]
mod tests;
