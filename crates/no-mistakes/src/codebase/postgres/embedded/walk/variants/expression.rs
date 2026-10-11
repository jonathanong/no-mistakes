use super::super::{resolve::compose, ScopeVisitor};
use super::{combine, template, Recovered, ValueKind};
use crate::codebase::postgres::embedded::{source_positions, tags};
use crate::codebase::ts_source::{byte_offset_to_line, unwrap_ts_wrappers};
use oxc_ast::ast::{BinaryOperator, Expression};
use oxc_span::GetSpan;

pub(super) fn recover(
    visitor: &ScopeVisitor<'_>,
    expr: &Expression<'_>,
    depth: u8,
    constraints: &[(u64, u32)],
) -> Option<Vec<Recovered>> {
    if depth == 0 {
        return None;
    }
    let expr = unwrap_ts_wrappers(expr);
    let line = byte_offset_to_line(visitor.source, expr.span().start as usize);
    match expr {
        Expression::NullLiteral(_) => {
            let mut value = Recovered::empty(line);
            value.value = ValueKind::Null;
            Some(vec![value])
        }
        Expression::BooleanLiteral(value) => {
            let mut scalar = Recovered::empty(line);
            scalar.value = ValueKind::Boolean(value.value);
            Some(vec![scalar])
        }
        Expression::Identifier(id) => visitor.lookup(id.name.as_str())?.variants.map(|values| {
            values
                .into_iter()
                .filter(|value| {
                    !value.choices.iter().any(|(id, arm)| {
                        constraints
                            .iter()
                            .any(|(other, choice)| id == other && arm != choice)
                    })
                })
                .collect()
        }),
        Expression::ConditionalExpression(branch) => {
            super::conditional::recover(visitor, branch, depth, constraints)
        }
        Expression::LogicalExpression(branch) => {
            let result = super::logical::recover(visitor, branch, depth, line, constraints)?;
            Some(
                result
                    .into_iter()
                    .map(|mut value| {
                        value.enumerated = true;
                        value
                    })
                    .collect(),
            )
        }
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            combine(
                recover(visitor, &binary.left, depth - 1, constraints)?,
                recover(visitor, &binary.right, depth - 1, constraints)?,
            )
        }
        Expression::TaggedTemplateExpression(tagged)
            if tags::is_sql_tag(
                &tagged.tag,
                &mut |name| compose::tag_shadowed(name, visitor),
                visitor.functions.imported_sql_tags(),
            ) =>
        {
            template::recover(visitor, &tagged.quasi, depth - 1, constraints)
        }
        Expression::CallExpression(call) => {
            if let Some(recovered) = super::helper::recover(visitor, call, depth - 1, constraints) {
                recovered
            } else {
                template::call(visitor, call, depth - 1, constraints).or_else(|| {
                    if compose::contains_parameter_helper(expr, visitor) {
                        None
                    } else {
                        literal(visitor, expr, line)
                    }
                })
            }
        }
        _ => literal(visitor, expr, line),
    }
}

fn literal(visitor: &ScopeVisitor<'_>, expr: &Expression<'_>, line: u32) -> Option<Vec<Recovered>> {
    let sql = compose::static_fragment(expr, visitor)?;
    Some(vec![Recovered {
        value: ValueKind::Sql,
        origins: source_positions::origins::expression(expr, visitor.source, &sql),
        sql,
        line,
        positions: source_positions::for_expression(
            expr,
            visitor.source,
            expr.span().start as usize,
            line,
        ),
        fragment: tags::fragments::is_sql_fragment(
            expr,
            &mut |name| compose::tag_shadowed(name, visitor),
            visitor.functions.imported_sql_tags(),
        ),
        choices: Vec::new(),
        append_sites: Vec::new(),
        enumerated: false,
    }])
}
