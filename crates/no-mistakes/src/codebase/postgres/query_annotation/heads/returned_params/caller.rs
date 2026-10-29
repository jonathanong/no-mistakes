//! All bounded branches start with one fresh inline native factory argument.
use super::{Leaf, Root};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::Expression;
use oxc_span::GetSpan;
mod lifetime;
mod projection;
mod wanted;
pub(in super::super) use projection::projected;
pub(in super::super) use wanted::extend as extend_wanted;
pub(in super::super) fn root<'a>(expression: &'a Expression<'a>) -> Option<Root> {
    let span = expression.span();
    let mut pending = vec![(expression, 0)];
    let mut leaves = Vec::new();
    let mut joined = None;
    while let Some((mut expression, mut appends)) = pending.pop() {
        loop {
            expression = unwrap_ts_wrappers(expression);
            if let Expression::ConditionalExpression(branch) = expression {
                if leaves.len() + pending.len() + 2 > 8 {
                    return None;
                }
                pending.push((&branch.alternate, appends));
                pending.push((&branch.consequent, appends));
                break;
            }
            let Expression::CallExpression(call) = expression else {
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
            if call.optional
                || !(1..=2).contains(&call.arguments.len())
                || call
                    .arguments
                    .iter()
                    .any(|argument| argument.as_expression().is_none())
            {
                return None;
            }
            let Expression::Identifier(callee) = unwrap_ts_wrappers(&call.callee) else {
                return None;
            };
            let first = call.arguments[0].as_expression()?;
            let Expression::TaggedTemplateExpression(tagged) = unwrap_ts_wrappers(first) else {
                return None;
            };
            if !tagged.quasi.expressions.is_empty() || tagged.quasi.quasis.len() != 1 {
                return None;
            }
            let Expression::Identifier(tag) = unwrap_ts_wrappers(&tagged.tag) else {
                return None;
            };
            // Factories and outer append links both continue the first native head.
            let proof = super::super::syntax::head(first).proof(true);
            if proof == super::super::HeadProof::Unproven
                || joined.is_some_and(|previous| previous != proof)
            {
                return None;
            }
            joined = Some(proof);
            leaves.push(Leaf {
                callee: (callee.span.start, callee.span.end),
                tag: (tag.span.start, tag.span.end),
                call: (call.span.start, call.span.end),
            });
            break;
        }
    }
    Some(Root {
        leaves,
        span: (span.start, span.end),
        proof: joined?,
    })
}
