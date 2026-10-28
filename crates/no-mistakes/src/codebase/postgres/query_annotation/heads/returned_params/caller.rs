//! Fresh inline native tag at the first argument of a direct helper call.
use super::Root;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::Expression;

pub(in super::super) fn root<'a>(mut expression: &'a Expression<'a>) -> Option<Root> {
    let mut appends = 0;
    loop {
        let Expression::CallExpression(call) = unwrap_ts_wrappers(expression) else {
            return None;
        };
        if let Some(receiver) = super::super::append::prefix_receiver(call) {
            if appends == 8 {
                return None;
            }
            expression = receiver;
            appends += 1;
            continue;
        }
        if call.optional || call.arguments.len() != 1 {
            return None;
        }
        let Expression::Identifier(callee) = unwrap_ts_wrappers(&call.callee) else {
            return None;
        };
        let Expression::TaggedTemplateExpression(first) =
            unwrap_ts_wrappers(call.arguments[0].as_expression()?)
        else {
            return None;
        };
        if !first.quasi.expressions.is_empty() || first.quasi.quasis.len() != 1 {
            return None;
        }
        let Expression::Identifier(tag) = unwrap_ts_wrappers(&first.tag) else {
            return None;
        };
        // The factory appends before returning, even without outer append links.
        let proof = super::super::syntax::head(call.arguments[0].as_expression()?).proof(true);
        return (proof != super::super::HeadProof::Unproven).then_some(Root {
            callee: (callee.span.start, callee.span.end),
            tag: (tag.span.start, tag.span.end),
            proof,
        });
    }
}
