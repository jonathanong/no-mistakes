use super::super::resolve::compose;
use super::super::ScopeVisitor;
use crate::codebase::postgres::embedded::{first_call_argument, tags, CallSqlFragment};
use crate::codebase::ts_source::{byte_offset_to_line, unwrap_ts_wrappers};
use oxc_ast::ast::{BinaryOperator, Expression};
use oxc_span::GetSpan;

mod concat;
mod relations;
use concat::{collect_addition, partial_atom, partial_parameterized_template, push_clean};
use relations::partial_templates;
pub(super) use relations::{closed_relation_names, try_relation_append};

pub(super) fn expr_fragments(
    visitor: &ScopeVisitor<'_>,
    expr: &Expression<'_>,
    context_sql: Option<&str>,
    append_site: Option<u32>,
) -> Vec<CallSqlFragment> {
    let mut out = Vec::new();
    collect(visitor, expr, context_sql, append_site, 8, &mut out);
    out
}

pub(super) fn is_sql_like_init(visitor: &ScopeVisitor<'_>, expr: &Expression<'_>) -> bool {
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(_)
        | Expression::TemplateLiteral(_)
        | Expression::TaggedTemplateExpression(_) => true,
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            is_sql_like_init(visitor, &binary.left) || is_sql_like_init(visitor, &binary.right)
        }
        Expression::CallExpression(call) => {
            matches!(
                unwrap_ts_wrappers(&call.callee),
                Expression::StaticMemberExpression(member) if member.property.name == "append"
            ) || tags::fragments::is_sql_fragment(
                expr,
                &mut |name| compose::tag_shadowed(name, visitor),
                visitor.functions.imported_sql_tags(),
            )
        }
        _ => false,
    }
}

fn collect(
    visitor: &ScopeVisitor<'_>,
    expr: &Expression<'_>,
    context_sql: Option<&str>,
    append_site: Option<u32>,
    depth: u8,
    out: &mut Vec<CallSqlFragment>,
) {
    if depth == 0 {
        return;
    }
    let expr = unwrap_ts_wrappers(expr);
    let span = expr.span();
    let line = byte_offset_to_line(visitor.source, span.start as usize);
    match expr {
        Expression::ConditionalExpression(cond) => {
            collect(
                visitor,
                &cond.consequent,
                context_sql,
                append_site,
                depth - 1,
                out,
            );
            collect(
                visitor,
                &cond.alternate,
                context_sql,
                append_site,
                depth - 1,
                out,
            );
        }
        Expression::LogicalExpression(logic) => {
            collect(
                visitor,
                &logic.left,
                context_sql,
                append_site,
                depth - 1,
                out,
            );
            collect(
                visitor,
                &logic.right,
                context_sql,
                append_site,
                depth - 1,
                out,
            );
        }
        Expression::TemplateLiteral(template) => {
            let prefixes: Vec<String> = context_sql.into_iter().map(str::to_string).collect();
            for sql in partial_templates(visitor, template, &prefixes, depth - 1) {
                push_clean(
                    out,
                    sql,
                    line,
                    (span.start, span.end),
                    context_sql,
                    append_site,
                );
            }
        }
        Expression::TaggedTemplateExpression(tagged)
            if tags::is_sql_tag(
                &tagged.tag,
                &mut |name| compose::tag_shadowed(name, visitor),
                visitor.functions.imported_sql_tags(),
            ) =>
        {
            let parameterized = visitor
                .functions
                .imported_sql_tags()
                .parameterizes_all(&tagged.tag);
            let sqls = if parameterized {
                vec![partial_parameterized_template(&tagged.quasi)]
            } else {
                let prefixes: Vec<String> = context_sql.into_iter().map(str::to_string).collect();
                partial_templates(visitor, &tagged.quasi, &prefixes, depth - 1)
            };
            for sql in sqls {
                push_clean(
                    out,
                    sql,
                    line,
                    (span.start, span.end),
                    context_sql,
                    append_site,
                );
            }
            for inner in &tagged.quasi.expressions {
                if tags::fragments::is_sql_fragment(
                    inner,
                    &mut |name| compose::tag_shadowed(name, visitor),
                    visitor.functions.imported_sql_tags(),
                ) {
                    collect(visitor, inner, context_sql, append_site, depth - 1, out);
                }
            }
        }
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            collect_addition(visitor, expr, context_sql, append_site, depth - 1, out);
        }
        Expression::CallExpression(call) => {
            if let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) {
                if member.property.name == "append" && call.arguments.len() == 1 {
                    let base_ctx = partial_atom(visitor, &member.object, depth - 1);
                    let effective_ctx = base_ctx.as_deref().or(context_sql);
                    collect(visitor, &member.object, context_sql, None, depth - 1, out);
                    if let Some(arg) = first_call_argument(call) {
                        collect(
                            visitor,
                            arg,
                            effective_ctx,
                            Some(call.span.start),
                            depth - 1,
                            out,
                        );
                    }
                    return;
                }
            }
            if let Some(sql) = partial_atom(visitor, expr, depth - 1) {
                push_clean(
                    out,
                    sql,
                    line,
                    (span.start, span.end),
                    context_sql,
                    append_site,
                );
            }
        }
        _ => {
            if let Some(sql) = partial_atom(visitor, expr, depth - 1) {
                push_clean(
                    out,
                    sql,
                    line,
                    (span.start, span.end),
                    context_sql,
                    append_site,
                );
            }
        }
    }
}
