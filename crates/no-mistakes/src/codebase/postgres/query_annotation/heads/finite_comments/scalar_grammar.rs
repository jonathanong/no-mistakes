//! Complete the first comment before permitting any opaque appended suffix.
use super::{Root, Span};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Expression, TaggedTemplateExpression};
fn text<'a>(value: &TaggedTemplateExpression<'a>) -> Option<std::borrow::Cow<'a, str>> {
    value
        .quasi
        .quasis
        .first()
        .filter(|part| part.value.cooked.is_some())
        .map(crate::ast::template_quasi_text)
}
pub(super) fn root<'a>(
    declaration: Span,
    function: Span,
    mut init: &'a Expression<'a>,
) -> Option<Root> {
    for _ in 0..=8 {
        let Expression::CallExpression(close) = unwrap_ts_wrappers(init) else {
            return None;
        };
        let receiver = super::super::append::prefix_receiver(close)?;
        if let Some(root) = complete(declaration, function, close, receiver) {
            return Some(root);
        }
        init = receiver;
    }
    None
}
fn complete<'a>(
    declaration: Span,
    function: Span,
    close: &'a oxc_ast::ast::CallExpression<'a>,
    receiver: &'a Expression<'a>,
) -> Option<Root> {
    let Expression::TaggedTemplateExpression(closing) =
        unwrap_ts_wrappers(close.arguments[0].as_expression()?)
    else {
        return None;
    };
    let after = text(closing)?;
    if !after.trim_start().starts_with("*/") {
        return None;
    }
    let Expression::Identifier(close_tag) = unwrap_ts_wrappers(&closing.tag) else {
        return None;
    };
    let Expression::CallExpression(label) = unwrap_ts_wrappers(receiver) else {
        return None;
    };
    let opening = super::super::append::prefix_receiver(label)?;
    let Expression::Identifier(reference) = unwrap_ts_wrappers(label.arguments[0].as_expression()?)
    else {
        return None;
    };
    let Expression::TaggedTemplateExpression(opening) = unwrap_ts_wrappers(opening) else {
        return None;
    };
    if !opening.quasi.expressions.is_empty() {
        return None;
    }
    let before = text(opening)?;
    if !before.trim_start().strip_prefix("/*")?.trim().is_empty() {
        return None;
    }
    let Expression::Identifier(tag) = unwrap_ts_wrappers(&opening.tag) else {
        return None;
    };
    let end = after.find("*/")? + 2;
    Some(Root {
        declaration,
        function,
        reference: (reference.span.start, reference.span.end),
        tag: (tag.span.start, tag.span.end),
        before: before.into_owned(),
        after: after[..end].to_string(),
        scalar: Some((close_tag.span.start, close_tag.span.end)),
    })
}
