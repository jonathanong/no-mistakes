use super::super::{ArrayGuard, Projector};
mod binding;
use oxc_ast::{ast::*, AstKind};
use oxc_span::GetSpan;
use oxc_syntax::{
    node::NodeId,
    operator::{BinaryOperator, LogicalOperator},
};

pub(super) fn collect<'a>(projector: &mut Projector<'a, '_>, node: NodeId) -> Vec<ArrayGuard> {
    let semantic = projector.semantic;
    let span = semantic.nodes().kind(node).span();
    let mut guards = Vec::new();
    for ancestor in semantic.nodes().ancestors(node).take(32) {
        match ancestor.kind() {
            AstKind::IfStatement(branch)
                if branch.consequent.span().start <= span.start
                    && span.end <= branch.consequent.span().end =>
            {
                condition(projector, &branch.test, &mut guards, 0);
            }
            AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => {
                guards.retain(|guard| guard.owner == ancestor.kind().span().start);
                return guards;
            }
            _ => {}
        }
    }
    Vec::new()
}

fn condition<'a>(
    projector: &mut Projector<'a, '_>,
    expr: &Expression<'a>,
    out: &mut Vec<ArrayGuard>,
    depth: usize,
) {
    if depth >= 8 {
        return;
    }
    match expr.get_inner_expression() {
        Expression::LogicalExpression(logic) if logic.operator == LogicalOperator::And => {
            condition(projector, &logic.left, out, depth + 1);
            condition(projector, &logic.right, out, depth + 1);
        }
        Expression::BinaryExpression(binary)
            if binary.operator == BinaryOperator::StrictEquality =>
        {
            for (candidate, literal) in
                [(&binary.left, &binary.right), (&binary.right, &binary.left)]
            {
                let expected = match literal.get_inner_expression() {
                    Expression::StringLiteral(s) => {
                        serde_json::Value::String(crate::ast::js_literal(s).into_owned())
                    }
                    Expression::NumericLiteral(n) => {
                        let Some(n) = serde_json::Number::from_f64(n.value) else {
                            continue;
                        };
                        serde_json::Value::Number(n)
                    }
                    _ => continue,
                };
                if let Some(mut guard) = binding::guard(projector, candidate) {
                    guard.expected = expected;
                    out.push(guard);
                }
            }
        }
        _ => {}
    }
}
