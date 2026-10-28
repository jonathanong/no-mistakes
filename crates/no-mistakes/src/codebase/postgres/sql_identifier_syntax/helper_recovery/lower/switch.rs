//! A bounded switch is a sequence of exclusive equality tests over one selector.
use super::{Expr, Scope, Stmt};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Expression, Statement, SwitchStatement};
use oxc_span::GetSpan;

pub(super) fn pure_throw(expr: &Expression<'_>) -> bool {
    matches!(
        unwrap_ts_wrappers(expr),
        Expression::StringLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::Identifier(_)
    )
}

pub(super) fn lower(scope: &mut Scope<'_>, node: &SwitchStatement<'_>) -> Vec<Stmt> {
    build(scope, node).unwrap_or_else(|| vec![Stmt::Opaque])
}

fn build(scope: &mut Scope<'_>, node: &SwitchStatement<'_>) -> Option<Vec<Stmt>> {
    let selector = scope.expression(&node.discriminant);
    if !matches!(selector, Expr::Ident(_) | Expr::Lit(_) | Expr::Member(_, _)) {
        return None;
    }
    let mut arms: Vec<(Vec<Expr>, Stmt)> = Vec::new();
    let mut labels = Vec::new();
    let mut seen = Vec::new();
    let mut default = None;
    for case in &node.cases {
        let Some(test) = &case.test else {
            if default.is_some() || !labels.is_empty() {
                return None;
            }
            default = Some(default_path(scope, &case.consequent)?);
            continue;
        };
        if default.is_some() {
            return None;
        }
        let Expression::StringLiteral(value) = unwrap_ts_wrappers(test) else {
            return None;
        };
        let label = crate::ast::js_literal(value).into_owned();
        if seen.contains(&label) {
            return None;
        }
        seen.push(label);
        labels.push(scope.expression(test));
        if !case.consequent.is_empty() {
            let [Statement::ReturnStatement(ret)] = case.consequent.as_slice() else {
                return None;
            };
            let argument = ret.argument.as_ref()?;
            let value = scope.expression(argument);
            if !matches!(&value, Expr::Lit(_))
                && !matches!(&value, Expr::Sql(_, holes, _) if holes.is_empty())
            {
                return None;
            }
            arms.push((
                std::mem::take(&mut labels),
                Stmt::Return(Some(value), scope.mark(argument.span())),
            ));
        }
    }
    if !labels.is_empty() || arms.is_empty() {
        return None;
    }
    let mut tail = default?;
    let slot = format!("switch:{}", node.span.start);
    for (labels, result) in arms.into_iter().rev() {
        for label in labels.into_iter().rev() {
            let test = Expr::Eq(
                Box::new(Expr::Ident(slot.clone())),
                Box::new(label),
                true,
                false,
            );
            tail = vec![Stmt::If(test, vec![result.clone()], tail)];
        }
    }
    let mut body = vec![Stmt::Let(slot, selector)];
    body.extend(tail);
    Some(body)
}

fn default_path(scope: &mut Scope<'_>, statements: &[Statement<'_>]) -> Option<Vec<Stmt>> {
    match statements {
        [Statement::ThrowStatement(throw)] => Some(vec![Stmt::Expr(scope.raise(&throw.argument))]),
        [Statement::ReturnStatement(ret)] => {
            let Expression::CallExpression(call) = unwrap_ts_wrappers(ret.argument.as_ref()?)
            else {
                return None;
            };
            if call.optional
                || !matches!(unwrap_ts_wrappers(&call.callee), Expression::Identifier(_))
                || !call
                    .arguments
                    .iter()
                    .all(|arg| arg.as_expression().is_some_and(pure_throw))
            {
                return None;
            }
            let call = scope.expression(ret.argument.as_ref()?);
            if !matches!(call, Expr::Call(_, _)) {
                return None;
            }
            // If the prepared callee returns, the default is not an abrupt path.
            Some(vec![Stmt::Expr(call), Stmt::Opaque])
        }
        _ => None,
    }
}
