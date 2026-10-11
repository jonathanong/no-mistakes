use super::super::{resolve::compose, ScopeVisitor};
use super::{combine, expression, Recovered, ValueKind};
use crate::codebase::postgres::embedded::{
    first_call_argument, placeholders, source_positions, tags,
};
use crate::codebase::ts_source::{byte_offset_to_line, unwrap_ts_wrappers};
use oxc_ast::ast::{CallExpression, Expression, TemplateLiteral};
use oxc_span::GetSpan;

pub(super) fn recover(
    visitor: &ScopeVisitor<'_>,
    template: &TemplateLiteral<'_>,
    depth: u8,
    constraints: &[(u64, u32)],
) -> Option<Vec<Recovered>> {
    if template
        .expressions
        .iter()
        .take(template.expressions.len().saturating_sub(1))
        .any(|expression| visitor.expression_mutates_builder(expression))
    {
        return None;
    }
    let origin = byte_offset_to_line(visitor.source, template.span.start as usize);
    let mut versions = vec![Recovered::empty(origin)];
    for (index, quasi) in template.quasis.iter().enumerate() {
        let line = byte_offset_to_line(visitor.source, quasi.span.start as usize);
        let raw = &visitor.source[quasi.span.start as usize..quasi.span.end as usize];
        let decoded = tags::quasi_text(quasi, false);
        let piece = Recovered {
            value: ValueKind::Sql,
            sql: decoded.into(),
            origins: source_positions::origins::piece(
                raw,
                decoded,
                quasi.span.start,
                quasi.value.cooked.is_none(),
            ),
            line,
            positions: source_positions::piece(raw, decoded, line, quasi.value.cooked.is_none()),
            fragment: true,
            choices: Vec::new(),
            enumerated: false,
        };
        versions = combine(versions, vec![piece])?;
        if let Some(expr) = template.expressions.get(index) {
            let is_fragment = tags::fragments::is_sql_fragment(
                expr,
                &mut |name| compose::tag_shadowed(name, visitor),
                visitor.functions.imported_sql_tags(),
            );
            let pieces = if is_fragment {
                expression::recover(visitor, expr, depth, constraints)?
                    .into_iter()
                    .map(|mut value| {
                        if !value.fragment {
                            value.sql = placeholders::internal_placeholder(1);
                            value.origins = vec![expr.span().start; value.sql.len()];
                            value.positions.clear();
                            value.line =
                                byte_offset_to_line(visitor.source, expr.span().start as usize);
                        }
                        value.value = ValueKind::Sql;
                        value
                    })
                    .collect()
            } else {
                let line = byte_offset_to_line(visitor.source, expr.span().start as usize);
                let mut value = Recovered::empty(line);
                value.sql = placeholders::internal_placeholder(1);
                value.origins = vec![expr.span().start; value.sql.len()];
                vec![value]
            };
            versions = combine(versions, pieces)?;
            if is_fragment {
                for version in &mut versions {
                    version.enumerated = true;
                }
            }
        }
    }
    for version in &mut versions {
        version.fragment = true;
    }
    Some(versions)
}

pub(super) fn call(
    visitor: &ScopeVisitor<'_>,
    call: &CallExpression<'_>,
    depth: u8,
    constraints: &[(u64, u32)],
) -> Option<Vec<Recovered>> {
    let trusted = |expr: &Expression<'_>| {
        tags::is_sql_tag(
            expr,
            &mut |name| compose::tag_shadowed(name, visitor),
            visitor.functions.imported_sql_tags(),
        )
    };
    if let Some((object, property)) = member(&call.callee) {
        if property == "append" {
            let prefix = expression::recover(visitor, object, depth, constraints)?;
            if !prefix.iter().all(|value| value.fragment) {
                return None;
            }
            return combine(
                prefix,
                expression::recover(visitor, first_call_argument(call)?, depth, constraints)?,
            );
        }
        if trusted(object) && property == "raw" {
            let arg = first_call_argument(call)?;
            // raw identifiers are opaque even if bound to a static string.
            if !matches!(
                unwrap_ts_wrappers(arg),
                Expression::StringLiteral(_) | Expression::TemplateLiteral(_)
            ) {
                return None;
            }
            let mut values = expression::recover(visitor, arg, depth, constraints)?;
            for value in &mut values {
                value.fragment = true;
            }
            return Some(values);
        }
        if trusted(object) && property == "join" {
            return join(visitor, call, depth, constraints);
        }
    }
    if trusted(&call.callee) {
        let mut result =
            expression::recover(visitor, first_call_argument(call)?, depth, constraints)?;
        for value in &mut result {
            value.fragment = true;
        }
        return Some(result);
    }
    None
}

fn join(
    visitor: &ScopeVisitor<'_>,
    call: &CallExpression<'_>,
    depth: u8,
    constraints: &[(u64, u32)],
) -> Option<Vec<Recovered>> {
    let Expression::ArrayExpression(array) = unwrap_ts_wrappers(first_call_argument(call)?) else {
        return None;
    };
    let separator = match call.arguments.get(1) {
        Some(arg) => expression::recover(visitor, arg.as_expression()?, depth, constraints)?,
        None => vec![Recovered::empty(byte_offset_to_line(
            visitor.source,
            call.span.start as usize,
        ))],
    };
    let mut result = vec![Recovered::empty(byte_offset_to_line(
        visitor.source,
        call.span.start as usize,
    ))];
    for (index, element) in array.elements.iter().enumerate() {
        if index > 0 {
            result = combine(result, separator.clone())?;
        }
        result = combine(
            result,
            expression::recover(visitor, element.as_expression()?, depth, constraints)?,
        )?;
    }
    for value in &mut result {
        value.fragment = true;
    }
    Some(result)
}

fn member<'a>(expr: &'a Expression<'a>) -> Option<(&'a Expression<'a>, &'a str)> {
    match unwrap_ts_wrappers(expr) {
        Expression::StaticMemberExpression(member) => {
            Some((&member.object, member.property.name.as_str()))
        }
        Expression::ComputedMemberExpression(member) => {
            let Expression::StringLiteral(property) = unwrap_ts_wrappers(&member.expression) else {
                return None;
            };
            Some((&member.object, property.value.as_str()))
        }
        _ => None,
    }
}
