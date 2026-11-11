//! Argument text and path origins for one call.
use super::super::path_origin::callee_root;
use super::super::{ArgumentText, PathAnchor, PathOrigin, PathProvenance, Projector};
use super::argument_source::ArgumentSource;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

/// Arguments after this index are never recorded.
pub(super) const MAX_ARGUMENTS: usize = 8;

/// Argument text is cut at this many bytes, on a char boundary.
const ARG_TEXT_LIMIT: usize = 80;

pub(super) fn record_arg_text(
    source: &ArgumentSource<'_>,
    call: &CallExpression<'_>,
    out: &mut PathProvenance,
    members: Vec<ArgumentText>,
) {
    let count = (call.arguments.len() <= 32
        && call
            .arguments
            .iter()
            .all(|arg| arg.as_expression().is_some()))
    .then_some(call.arguments.len());
    out.arg_counts.insert(call.span.start, count);
    let mut texts: Vec<_> = call
        .arguments
        .iter()
        .take(MAX_ARGUMENTS)
        .enumerate()
        .map(|(index, argument)| argument_text(source, index, None, argument.span()))
        .filter(|argument| !argument.text.is_empty())
        .collect();
    texts.extend(members);
    if !texts.is_empty() {
        out.arg_text.insert(call.span.start, texts);
    }
}

pub(super) fn argument_text(
    source: &ArgumentSource<'_>,
    index: usize,
    property: Option<String>,
    span: oxc_span::Span,
) -> ArgumentText {
    let mut text = String::new();
    for word in crate::ast::span_text(source.text, span).split_whitespace() {
        if !text.is_empty() {
            text.push(' ');
        }
        text.push_str(word);
        if text.len() >= ARG_TEXT_LIMIT {
            break;
        }
    }
    text.truncate(text.floor_char_boundary(ARG_TEXT_LIMIT));
    ArgumentText {
        index,
        property,
        line: source.line(span.start),
        text,
    }
}

/// `sync.call(...)`, `.apply`, and `.bind` shift argument positions.
pub(super) fn rebinds_callee(callee: &Expression<'_>) -> bool {
    matches!(callee.get_inner_expression(), Expression::StaticMemberExpression(member)
        if matches!(member.property.name.as_str(), "call" | "apply" | "bind"))
}

/// Only anchors that can prove an owned or fixture root are kept, which bounds memory.
pub(super) fn argument_paths<'a>(
    projector: &mut Projector<'a, '_>,
    call: &CallExpression<'a>,
    out: &mut PathProvenance,
) -> Vec<(usize, PathOrigin)> {
    let mut paths = Vec::new();
    let mut unproven = Vec::new();
    for index in 0..MAX_ARGUMENTS {
        let Some(origin) = projector.argument_origin(call, index) else {
            break;
        };
        if kept(&origin) {
            paths.push((index, origin));
        } else {
            unproven.push((index, origin));
        }
    }
    if !unproven.is_empty() {
        out.arg_unproven.insert(call.span.start, unproven);
    }
    paths
}

fn kept(origin: &PathOrigin) -> bool {
    origin.atoms().iter().any(|origin| {
        matches!(
            origin.anchor,
            PathAnchor::Temp | PathAnchor::Module | PathAnchor::Cwd
        ) || origin.param.is_some()
            || origin.env.is_some()
                || origin.getter.is_some()
    })
}

/// `it(...)` / `test(...)` (including `.only` / `.skip`) with a function body.
pub(super) fn test_callback(call: &CallExpression<'_>) -> Option<(u32, u32)> {
    if !matches!(callee_root(&call.callee)?, "it" | "test") {
        return None;
    }
    call.arguments.iter().find_map(|argument| {
        match argument.as_expression()?.get_inner_expression() {
            Expression::ArrowFunctionExpression(function) => {
                Some((function.span.start, function.span.end))
            }
            Expression::FunctionExpression(function) => {
                Some((function.span.start, function.span.end))
            }
            _ => None,
        }
    })
}
