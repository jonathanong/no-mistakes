use super::csp_strings::{eval_static_string, static_template_text};
use super::super::{
    CollectedOrigins, CspDirective, OriginRef, ParsedOrigin, UnanalyzableReason, UnanalyzableRef,
};
use crate::codebase::ts_source::{line_number, unwrap_ts_wrappers};
use oxc_ast::ast::{BinaryOperator, Expression};
use oxc_span::GetSpan;
use std::borrow::Cow;

pub(super) fn record_url_expr(
    collected: &mut CollectedOrigins,
    source: &str,
    expr: &Expression<'_>,
    directive: CspDirective,
) {
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(lit) => {
            record_static_url(
                collected,
                source,
                lit.span.start,
                directive,
                &crate::ast::js_literal(lit),
            );
        }
        Expression::TemplateLiteral(tpl) => match static_template_text(tpl) {
            Some(text) => record_static_url(collected, source, tpl.span.start, directive, &text),
            None => {
                let prefix = tpl
                    .quasis
                    .first()
                    .filter(|q| q.value.cooked.is_some())
                    .map(crate::ast::template_quasi_text);
                record_prefix_or_dynamic(collected, source, tpl.span.start, directive, prefix);
            }
        },
        Expression::ConditionalExpression(cond) => {
            record_url_expr(collected, source, &cond.consequent, directive);
            record_url_expr(collected, source, &cond.alternate, directive);
        }
        Expression::BinaryExpression(bin) if bin.operator == BinaryOperator::Addition => {
            match eval_static_string(expr) {
                Some(text) => {
                    record_static_url(collected, source, bin.span.start, directive, &text);
                }
                None => record_prefix_or_dynamic(
                    collected,
                    source,
                    bin.span.start,
                    directive,
                    eval_static_string(&bin.left),
                ),
            }
        }
        other => record_unanalyzable(
            collected,
            source,
            other.span().start,
            UnanalyzableReason::DynamicUrl(directive),
        ),
    }
}

pub(super) fn record_static_url(
    collected: &mut CollectedOrigins,
    source: &str,
    span_start: u32,
    directive: CspDirective,
    url: &str,
) {
    if let Some(origin) = ParsedOrigin::parse(url) {
        collected.refs.push(OriginRef {
            line: line_number(source, span_start),
            directive,
            origin,
        });
    } else if ParsedOrigin::is_network_like_url(url) {
        record_unanalyzable(
            collected,
            source,
            span_start,
            UnanalyzableReason::UnparsedUrl(directive),
        );
    }
}

pub(super) fn record_unanalyzable(
    collected: &mut CollectedOrigins,
    source: &str,
    span: u32,
    reason: UnanalyzableReason,
) {
    collected.unanalyzable.push(UnanalyzableRef {
        line: line_number(source, span),
        reason,
    });
}

fn record_prefix_or_dynamic(
    collected: &mut CollectedOrigins,
    source: &str,
    span: u32,
    dir: CspDirective,
    prefix: Option<Cow<'_, str>>,
) {
    if let Some(prefix) = prefix {
        if let Some(origin_url) = static_origin_prefix(&prefix) {
            record_static_url(collected, source, span, dir, &origin_url);
            return;
        }
        if is_guaranteed_non_third_party_prefix(&prefix) {
            return;
        }
    }
    record_unanalyzable(collected, source, span, UnanalyzableReason::DynamicUrl(dir));
}

fn static_origin_prefix(prefix: &str) -> Option<String> {
    let trimmed = prefix.trim_start();
    let after_scheme = if let Some(rest) = trimmed.strip_prefix("//") {
        rest
    } else {
        let (scheme, rest) = trimmed.split_once("://")?;
        if !matches!(
            scheme.to_ascii_lowercase().as_str(),
            "http" | "https" | "ws" | "wss"
        ) {
            return None;
        }
        rest
    };
    after_scheme
        .contains(['/', '?', '#'])
        .then(|| trimmed.to_owned())
}

fn is_guaranteed_non_third_party_prefix(prefix: &str) -> bool {
    let trimmed = prefix.trim_start();
    if (trimmed.starts_with('/') && trimmed.len() >= 2 && !trimmed.starts_with("//"))
        || trimmed.starts_with("./")
        || trimmed.starts_with("../")
        || trimmed.starts_with('?')
        || trimmed.starts_with('#')
    {
        return true;
    }
    trimmed.split_once(':').is_some_and(|(scheme, _)| {
        !scheme.is_empty()
            && scheme
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'-' | b'.'))
            && !matches!(
                scheme.to_ascii_lowercase().as_str(),
                "http" | "https" | "ws" | "wss"
            )
    })
}

pub(super) fn is_global_or_bare(callee: &Expression<'_>, names: &[&str]) -> bool {
    match unwrap_ts_wrappers(callee) {
        Expression::Identifier(id) => names.contains(&id.name.as_str()),
        Expression::StaticMemberExpression(member) if !member.optional => {
            names.contains(&member.property.name.as_str())
                && matches!(
                    unwrap_ts_wrappers(&member.object),
                    Expression::Identifier(obj)
                        if matches!(obj.name.as_str(), "window" | "globalThis" | "self")
                )
        }
        _ => false,
    }
}
