use super::super::tags;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{BinaryOperator, Expression};
use oxc_span::GetSpan;

/// Merge source positions for `left + right` static SQL.
///
/// Concatenation drops the physical newline between operands, so each operand
/// keeps its own source line instead of the declaration line.
pub(super) fn try_append(
    expr: &Expression<'_>,
    source: &str,
    anchor: usize,
    anchor_line: u32,
    out: &mut super::Positions,
) -> bool {
    let saved = snapshot(out);
    if append_expr(expr, source, anchor, anchor_line, out) {
        return true;
    }
    *out = saved;
    false
}

fn append_expr(
    expr: &Expression<'_>,
    source: &str,
    anchor: usize,
    anchor_line: u32,
    out: &mut super::Positions,
) -> bool {
    let expr = unwrap_ts_wrappers(expr);
    let start = expr.span().start as usize;
    if start < anchor {
        return false;
    }
    let line = anchor_line + super::newlines(&source[anchor..start]);
    match expr {
        Expression::StringLiteral(literal) => {
            let raw = &source[literal.span.start as usize + 1..literal.span.end as usize - 1];
            out.append(raw, literal.value.as_str(), line, false);
            true
        }
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            super::template_positions(template, source, line, out, false);
            true
        }
        Expression::TemplateLiteral(_) => false,
        Expression::TaggedTemplateExpression(tagged) => {
            let line = line
                + super::newlines(
                    &source[tagged.span.start as usize..tagged.quasi.span.start as usize],
                );
            super::template_positions(
                &tagged.quasi,
                source,
                line,
                out,
                tags::is_string_raw_tag_spelling(&tagged.tag),
            );
            true
        }
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            append_expr(&binary.left, source, anchor, anchor_line, out)
                && append_expr(&binary.right, source, anchor, anchor_line, out)
        }
        _ => false,
    }
}

fn snapshot(out: &super::Positions) -> super::Positions {
    super::Positions {
        positions: out.positions.clone(),
        line: out.line,
        column: out.column,
        origin: out.origin,
        placeholder_offset: out.placeholder_offset,
    }
}
