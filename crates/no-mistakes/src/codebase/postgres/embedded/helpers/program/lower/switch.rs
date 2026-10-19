//! Literal `switch` returns, a proven throwing `default`, and throw guards.

#[rustfmt::skip]
mod shape {
    use super::super::super::{Expr, Stmt, SwitchArm};
    use super::super::{expr, Scope};
    use crate::codebase::ts_source::unwrap_ts_wrappers;
    use oxc_ast::ast::{Argument, CallExpression, Expression, FunctionBody, IfStatement, LogicalOperator, Statement, SwitchStatement, UnaryOperator};
    use oxc_span::GetSpan;
    use std::collections::HashSet;

    /// Same-file functions whose bodies always throw. The name is not evidence.
    pub(crate) fn names<'a>(functions: impl IntoIterator<Item = (&'a str, &'a FunctionBody<'a>)>) -> HashSet<String> {
        let functions: Vec<_> = functions.into_iter().collect();
        let mut proven = HashSet::new();
        loop {
            let count = proven.len();
            for (name, body) in &functions {
                if sequence(&body.statements, &proven) { proven.insert((*name).to_string()); }
            }
            if proven.len() == count { return proven; }
        }
    }

    /// A statement before `throw` can write SQL, so it is not proof.
    fn sequence(stmts: &[Statement<'_>], proven: &HashSet<String>) -> bool {
        for statement in stmts {
            let throws = match statement {
                Statement::EmptyStatement(_) => continue,
                Statement::ThrowStatement(_) => true,
                Statement::BlockStatement(block) => return sequence(&block.body, proven),
                Statement::ExpressionStatement(expr) => call_throws(&expr.expression, proven),
                Statement::ReturnStatement(ret) => ret.argument.as_ref().is_some_and(|arg| call_throws(arg, proven)),
                Statement::IfStatement(ifs) => {
                    return sequence(std::slice::from_ref(&ifs.consequent), proven)
                        && ifs.alternate.as_ref().is_some_and(|alt| sequence(std::slice::from_ref(alt), proven));
                }
                _ => false,
            };
            return throws;
        }
        false
    }
    fn call_throws(expr: &Expression<'_>, proven: &HashSet<String>) -> bool {
        let Expression::CallExpression(call) = unwrap_ts_wrappers(expr) else { return false };
        let Expression::Identifier(ident) = unwrap_ts_wrappers(&call.callee) else { return false };
        !call.optional
            && proven.contains(ident.name.as_str())
            && call.arguments.iter().all(|arg| !matches!(arg, Argument::SpreadElement(_)))
    }

    pub(crate) fn skippable(statement: &IfStatement<'_>, proven: &HashSet<String>) -> bool {
        statement.alternate.is_none()
            && sequence(std::slice::from_ref(&statement.consequent), proven)
            && opaque_check(&statement.test)
    }

    fn opaque_check(expr: &Expression<'_>) -> bool {
        match unwrap_ts_wrappers(expr) {
            Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::LogicalNot => opaque_check(&unary.argument),
            Expression::LogicalExpression(logical) if logical.operator != LogicalOperator::Coalesce => {
                opaque_check(&logical.left) && opaque_check(&logical.right)
            }
            Expression::CallExpression(call) => member_call(call),
            _ => false,
        }
    }

    fn member_call(call: &CallExpression<'_>) -> bool {
        if call.optional { return false; }
        let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) else { return false };
        member.property.name != "append"
            && matches!(unwrap_ts_wrappers(&member.object), Expression::Identifier(_))
            && call.arguments.iter().all(|arg| arg.as_expression().is_some_and(ident))
    }

    fn ident(expr: &Expression<'_>) -> bool {
        matches!(unwrap_ts_wrappers(expr), Expression::Identifier(_))
    }
    pub(crate) fn lower(scope: &mut Scope<'_, '_>, statement: &SwitchStatement<'_>) -> Stmt {
        let Some(discriminant) = selector(scope, &statement.discriminant) else { return Stmt::Opaque };
        let mut arms: Vec<SwitchArm> = Vec::new();
        let mut pending = Vec::new();
        let mut threw = false;
        for case in &statement.cases {
            let Some(test) = &case.test else {
                if !sequence(&case.consequent, scope.ctx.throws) { return Stmt::Opaque; }
                pending.clear();
                threw = true;
                continue;
            };
            let Some(label) = string_label(test) else { return Stmt::Opaque };
            if pending.contains(&label) || arms.iter().any(|arm| arm.cases.contains(&label)) {
                return Stmt::Opaque;
            }
            pending.push(label);
            if case.consequent.is_empty() { continue; }
            let Some(arm) = literal_return(scope, &case.consequent, &mut pending) else { return Stmt::Opaque };
            arms.push(arm);
        }
        if threw && pending.is_empty() && !arms.is_empty() { Stmt::Switch(discriminant, arms) } else { Stmt::Opaque }
    }

    /// A number evaluates as an unknown scalar and would keep every arm.
    fn selector(scope: &mut Scope<'_, '_>, expr: &Expression<'_>) -> Option<Expr> {
        let lowered = expr::lower(scope, expr);
        match lowered {
            Expr::Ident(_) | Expr::Lit(_) | Expr::Member(_, _) => Some(lowered),
            _ => None,
        }
    }

    fn string_label(test: &Expression<'_>) -> Option<String> {
        match unwrap_ts_wrappers(test) {
            Expression::StringLiteral(literal) => Some(literal.value.to_string()),
            _ => None,
        }
    }

    fn literal_return(scope: &mut Scope<'_, '_>, statements: &[Statement<'_>], labels: &mut Vec<String>) -> Option<SwitchArm> {
        let statements = match statements {
            [Statement::BlockStatement(block)] => block.body.as_slice(),
            other => other,
        };
        let [Statement::ReturnStatement(ret)] = statements else { return None };
        let argument = ret.argument.as_ref()?;
        let value = expr::lower(scope, argument);
        if !matches!(value, Expr::Lit(_) | Expr::Sql(_, _)) { return None; }
        Some(SwitchArm { cases: std::mem::take(labels), value, mark: scope.mark(argument.span()) })
    }
}

pub(crate) use shape::names;
pub(super) use shape::{lower, skippable};
