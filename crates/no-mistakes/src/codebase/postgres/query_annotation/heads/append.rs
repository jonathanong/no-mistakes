//! Only a direct static append preserves the syntactic receiver's leading head.
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression};

pub(super) fn receiver<'a>(call: &'a CallExpression<'a>) -> Option<&'a Expression<'a>> {
    let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) else {
        return None;
    };
    (!call.optional && !member.optional && member.property.name == "append")
        .then_some(&member.object)
}

/// Check syntax only: interpolations are bind placeholders in tagged SQL.
/// Never allocate or compose suffix text to establish an annotation head.
pub(super) fn literal_receiver<'a>(call: &'a CallExpression<'a>) -> Option<&'a Expression<'a>> {
    let [argument] = call.arguments.as_slice() else {
        return None;
    };
    if !matches!(
        unwrap_ts_wrappers(argument.as_expression()?),
        Expression::StringLiteral(_) | Expression::TaggedTemplateExpression(_)
    ) {
        return None;
    }
    receiver(call)
}

/// An append chain used as a statement cannot expose the returned builder alias.
pub(super) fn reference<'a>(mut expression: &'a Expression<'a>) -> Option<super::Span> {
    let mut appended = false;
    loop {
        expression = unwrap_ts_wrappers(expression);
        let Expression::CallExpression(call) = expression else {
            break;
        };
        expression = literal_receiver(call)?;
        appended = true;
    }
    match expression {
        Expression::Identifier(id) if appended => Some((id.span.start, id.span.end)),
        _ => None,
    }
}
