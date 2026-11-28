use super::super::super::ScopeVisitor;
use super::super::{combine, expression, Recovered, ValueKind};
use super::{effects, piece};
use crate::codebase::postgres::embedded::helpers::sentinel;
use crate::codebase::postgres::embedded::{placeholders, source_positions};
use crate::codebase::ts_source::{byte_offset_to_line, unwrap_ts_wrappers};
use oxc_ast::ast::{Expression, TemplateLiteral};
use oxc_span::GetSpan;

/// Recover closed-union table-name alternatives when an interpolation sits in a relation slot.
pub(super) fn pieces(
    visitor: &ScopeVisitor<'_>,
    expr: &Expression<'_>,
    versions: &[Recovered],
    depth: u8,
    constraints: &[(u64, u32)],
) -> Option<Vec<Recovered>> {
    if depth == 0
        || versions.is_empty()
        || !versions
            .iter()
            .all(|version| sentinel::in_relation_slot(&version.sql, version.sql.len()))
    {
        return None;
    }
    let expr = unwrap_ts_wrappers(expr);
    let mut values =
        super::super::super::closed_values::recover_relation(visitor, expr).or_else(|| {
            if matches!(
                expr,
                Expression::ConditionalExpression(_)
                    | Expression::LogicalExpression(_)
                    | Expression::Identifier(_)
            ) {
                let recovered = expression::recover(visitor, expr, depth - 1, constraints)?;
                let is_union = recovered.len() > 1
                    || recovered
                        .iter()
                        .any(|value| value.enumerated || !value.choices.is_empty());
                is_union.then_some(recovered)
            } else {
                None
            }
        })?;
    if values.is_empty()
        || !values.iter().all(|value| {
            value.value == ValueKind::Sql
                && !value.fragment
                && !value.sql.trim().is_empty()
                && !value.sql.contains('\0')
                && !sentinel::contains(&value.sql)
        })
    {
        return None;
    }
    let line = byte_offset_to_line(visitor.source, expr.span().start as usize);
    let positions =
        source_positions::for_expression(expr, visitor.source, expr.span().start as usize, line);
    for value in &mut values {
        value.closed_domain = true;
        value.enumerated = true;
        if value.origins.len() != value.sql.len() {
            value.origins = vec![expr.span().start; value.sql.len()];
        }
        if value.positions.is_empty() {
            value.positions = positions.clone();
        }
    }
    Some(values)
}

/// Expand closed-union relation interpolations inside a `sql-template-strings` template.
/// Returns `None` when no relation slot expands, leaving ordinary templates untouched.
pub(in super::super) fn recover_parameterized(
    visitor: &ScopeVisitor<'_>,
    template: &TemplateLiteral<'_>,
    depth: u8,
    constraints: &[(u64, u32)],
) -> Option<Vec<Recovered>> {
    if depth == 0 || effects::mutates_captured_fragment(visitor, template) {
        return None;
    }
    let origin = byte_offset_to_line(visitor.source, template.span.start as usize);
    let mut versions = vec![Recovered::empty(origin)];
    let mut expanded_relation = false;
    let mut binds = 0usize;
    for (index, quasi) in template.quasis.iter().enumerate() {
        versions = combine(versions, vec![piece::quasi(visitor, quasi)])?;
        if let Some(expr) = template.expressions.get(index) {
            if let Some(relation_pieces) = pieces(visitor, expr, &versions, depth, constraints) {
                expanded_relation = true;
                versions = combine(versions, relation_pieces)?;
            } else if versions
                .iter()
                .all(|version| sentinel::in_relation_slot(&version.sql, version.sql.len()))
            {
                let line = byte_offset_to_line(visitor.source, expr.span().start as usize);
                let mut value = Recovered::empty(line);
                value.sql = placeholders::internal_placeholder(index + 1);
                value.origins = vec![expr.span().start; value.sql.len()];
                versions = combine(versions, vec![value])?;
            } else {
                binds += 1;
                let line = byte_offset_to_line(visitor.source, expr.span().start as usize);
                let mut value = Recovered::empty(line);
                value.sql = placeholders::internal_bind(binds);
                value.origins = vec![expr.span().start; value.sql.len()];
                versions = combine(versions, vec![value])?;
            }
        }
    }
    if !expanded_relation {
        return None;
    }
    for version in &mut versions {
        version.fragment = true;
        version.enumerated = true;
        version.closed_domain = true;
    }
    Some(versions)
}
