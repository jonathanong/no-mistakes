use crate::codebase::postgres::embedded::helpers::sentinel;
use crate::codebase::postgres::embedded::walk::{resolve::compose, ScopeVisitor};
use crate::codebase::postgres::embedded::{placeholders, tags, CallSqlFragment};
use crate::codebase::ts_source::{byte_offset_to_line, unwrap_ts_wrappers};
use oxc_ast::ast::{BinaryOperator, Expression, TemplateLiteral};
use oxc_span::GetSpan;

pub(super) fn collect_addition<'a>(
    visitor: &ScopeVisitor<'_>,
    expr: &'a Expression<'a>,
    context_sql: Option<&str>,
    append_site: Option<u32>,
    depth: u8,
    out: &mut Vec<CallSqlFragment>,
) {
    let mut leaves = Vec::new();
    flatten_add(expr, &mut leaves);
    let span = expr.span();
    let line = byte_offset_to_line(visitor.source, span.start as usize);
    let mut combined = String::new();
    let mut leading_static = String::new();
    let mut seen_opaque = false;
    let mut placeholders_used = 0usize;
    let mut trailing: Vec<(String, u32, (u32, u32))> = Vec::new();
    for leaf in leaves {
        let leaf_span = leaf.span();
        let leaf_line = byte_offset_to_line(visitor.source, leaf_span.start as usize);
        if let Some(piece) = partial_atom(visitor, leaf, depth) {
            combined.push_str(&piece);
            if !seen_opaque {
                leading_static.push_str(&piece);
            } else {
                trailing.push((piece, leaf_line, (leaf_span.start, leaf_span.end)));
            }
        } else {
            seen_opaque = true;
            placeholders_used += 1;
            combined.push_str(&placeholders::internal_placeholder(placeholders_used));
        }
    }
    if !combined.trim().is_empty() {
        push_clean(
            out,
            combined,
            line,
            (span.start, span.end),
            context_sql,
            append_site,
        );
    }
    if seen_opaque && !trailing.is_empty() {
        let effective_ctx = if !leading_static.trim().is_empty() {
            Some(leading_static.as_str())
        } else {
            context_sql
        };
        for (piece, leaf_line, leaf_span) in trailing {
            if piece.contains('*') {
                push_clean(out, piece, leaf_line, leaf_span, effective_ctx, append_site);
            }
        }
    }
}

fn flatten_add<'a>(expr: &'a Expression<'a>, out: &mut Vec<&'a Expression<'a>>) {
    let expr = unwrap_ts_wrappers(expr);
    if let Expression::BinaryExpression(binary) = expr {
        if binary.operator == BinaryOperator::Addition {
            flatten_add(&binary.left, out);
            flatten_add(&binary.right, out);
            return;
        }
    }
    out.push(expr);
}

pub(super) fn partial_template(
    visitor: &ScopeVisitor<'_>,
    template: &TemplateLiteral<'_>,
    depth: u8,
) -> String {
    let mut out = String::new();
    for (index, quasi) in template.quasis.iter().enumerate() {
        out.push_str(&tags::quasi_text(quasi, false));
        if let Some(expr) = template.expressions.get(index) {
            if let Some(piece) = partial_atom(visitor, expr, depth) {
                out.push_str(&piece);
            } else {
                out.push_str(&placeholders::internal_placeholder(index + 1));
            }
        }
    }
    out
}

pub(super) fn partial_atom(
    visitor: &ScopeVisitor<'_>,
    expr: &Expression<'_>,
    depth: u8,
) -> Option<String> {
    if depth == 0 {
        return None;
    }
    let expr = unwrap_ts_wrappers(expr);
    let text = match expr {
        Expression::StringLiteral(literal) => Some(crate::ast::js_literal(literal).into_owned()),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            crate::codebase::postgres::embedded::unpublished_sql_text(expr)
        }
        Expression::TemplateLiteral(template) => {
            Some(partial_template(visitor, template, depth - 1))
        }
        Expression::Identifier(ident) => {
            let binding = visitor.lookup(ident.name.as_str())?;
            if binding.kind == crate::codebase::postgres::embedded::EmbeddedSqlKind::Dynamic {
                None
            } else {
                binding.sql
            }
        }
        _ => compose::static_fragment(expr, visitor),
    }?;
    (!text.contains('\0') && !sentinel::contains(&text)).then_some(text)
}

pub(super) fn push_clean(
    out: &mut Vec<CallSqlFragment>,
    sql: String,
    line: u32,
    span: (u32, u32),
    context_sql: Option<&str>,
    append_site: Option<u32>,
) {
    if sql.trim().is_empty() || sql.contains('\0') || sentinel::contains(&sql) {
        return;
    }
    out.push(CallSqlFragment {
        sql,
        line,
        span,
        context_sql: context_sql.map(str::to_string),
        append_site,
    });
}
