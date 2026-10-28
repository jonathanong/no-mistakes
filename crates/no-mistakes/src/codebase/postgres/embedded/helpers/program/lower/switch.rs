//! Literal `switch` returns, a proven throwing `default`, and throw guards.

#[rustfmt::skip]
mod shape {
    use super::super::super::{Expr, Stmt, SwitchArm};
    use super::super::{expr, Scope};
    use crate::codebase::ts_source::unwrap_ts_wrappers;
    use oxc_ast::ast::{Argument, BindingPattern, Expression, FormalParameters, FunctionBody, Statement, SwitchStatement, VariableDeclarationKind};
    use oxc_span::GetSpan;
    use std::collections::HashSet;

    /// Same-file functions whose bodies always throw. The name is not evidence.
    /// A callee shadowed by the candidate's own parameters or body is not proof.
    pub(crate) fn names<'a>(functions: impl IntoIterator<Item = (&'a str, &'a FormalParameters<'a>, &'a FunctionBody<'a>)>) -> HashSet<String> {
        let functions: Vec<_> = functions
            .into_iter()
            .map(|(name, params, body)| (name, super::shadows(params, body), body))
            .collect();
        let mut proven = HashSet::new();
        loop {
            let count = proven.len();
            for (name, shadowed, body) in &functions {
                let throws = {
                    let throws = |callee: &str| proven.contains(callee) && !shadowed.contains(callee);
                    sequence(&body.statements, &throws)
                };
                if throws { proven.insert((*name).to_string()); }
            }
            if proven.len() == count { return proven; }
        }
    }

    /// A statement before `throw` can write SQL, so it is not proof.
    fn sequence<F: Fn(&str) -> bool>(stmts: &[Statement<'_>], throws: &F) -> bool {
        for statement in stmts {
            let throws = match statement {
                Statement::EmptyStatement(_) => continue,
                Statement::ThrowStatement(_) => true,
                Statement::BlockStatement(block) => return sequence(&block.body, throws),
                Statement::ExpressionStatement(expr) => call_throws(&expr.expression, throws),
                Statement::ReturnStatement(ret) => ret.argument.as_ref().is_some_and(|arg| call_throws(arg, throws)),
                Statement::IfStatement(ifs) => {
                    return sequence(std::slice::from_ref(&ifs.consequent), throws)
                        && ifs.alternate.as_ref().is_some_and(|alt| sequence(std::slice::from_ref(alt), throws));
                }
                _ => false,
            };
            return throws;
        }
        false
    }
    fn call_throws(expr: &Expression<'_>, throws: &impl Fn(&str) -> bool) -> bool {
        let Expression::CallExpression(call) = unwrap_ts_wrappers(expr) else { return false };
        let Expression::Identifier(ident) = unwrap_ts_wrappers(&call.callee) else { return false };
        !call.optional
            && throws(ident.name.as_str())
            && call.arguments.iter().all(|arg| !matches!(arg, Argument::SpreadElement(_)))
    }

    pub(crate) fn lower(scope: &mut Scope<'_, '_>, statement: &SwitchStatement<'_>) -> Stmt {
        let Some(discriminant) = selector(scope, &statement.discriminant) else { return Stmt::Opaque };
        let mut arms: Vec<SwitchArm> = Vec::new();
        let mut pending = Vec::new();
        // Labels that fell through into a throwing default. A later repeat is not that arm.
        let mut seen = Vec::new();
        let mut default = None;
        for case in &statement.cases {
            let Some(test) = &case.test else {
                let throws = |name: &str| scope.ctx.throws.contains(name) && !scope.is_rebound(name);
                let proven = sequence(&case.consequent, &throws) || inert_then_throw(scope, &case.consequent);
                let deferred = if proven { None } else { deferred_throw(scope, &case.consequent) };
                if !proven && deferred.is_none() { return Stmt::Opaque; }
                seen.append(&mut pending);
                default = Some(deferred);
                continue;
            };
            let Some(label) = string_label(test) else { return Stmt::Opaque };
            if pending.contains(&label) || seen.contains(&label) || arms.iter().any(|arm| arm.cases.contains(&label)) {
                return Stmt::Opaque;
            }
            pending.push(label);
            if case.consequent.is_empty() { continue; }
            let Some(arm) = literal_return(scope, &case.consequent, &mut pending) else { return Stmt::Opaque };
            arms.push(arm);
        }
        if let Some(default) = default.filter(|_| pending.is_empty() && !arms.is_empty()) {
            Stmt::Switch(discriminant, arms, default)
        } else { Stmt::Opaque }
    }

    /// Only a read of a parameter into a fresh const may precede a final throw.
    fn inert_then_throw(scope: &Scope<'_, '_>, statements: &[Statement<'_>]) -> bool {
        let statements = match statements {
            [Statement::BlockStatement(block)] => block.body.as_slice(),
            other => other,
        };
        let Some((last, prefix)) = statements.split_last() else { return false };
        matches!(last, Statement::ThrowStatement(_)) && prefix.iter().all(|statement| {
            let Statement::VariableDeclaration(declaration) = statement else { return false };
            if declaration.kind != VariableDeclarationKind::Const { return false; }
            let [declarator] = declaration.declarations.as_slice() else { return false };
            let BindingPattern::BindingIdentifier(_) = &declarator.id else { return false };
            let Some(init) = &declarator.init else { return false };
            let Expression::Identifier(value) = unwrap_ts_wrappers(init) else { return false };
            scope.params.iter().any(|param| param == value.name.as_str())
        })
    }

    /// A source-visible callee may prove abruptness only during prepared projection.
    fn deferred_throw(scope: &mut Scope<'_, '_>, statements: &[Statement<'_>]) -> Option<Expr> {
        let [Statement::ReturnStatement(ret)] = statements else { return None };
        let value = expr::lower(scope, ret.argument.as_ref()?);
        matches!(value, Expr::Call(..)).then_some(value)
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
            Expression::StringLiteral(literal) => Some(crate::ast::js_literal(literal).into_owned()),
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
        if !matches!(value, Expr::Lit(_) | Expr::Sql(..)) { return None; }
        Some(SwitchArm { cases: std::mem::take(labels), value, mark: scope.mark(argument.span()) })
    }
}

pub(super) use shape::lower;
pub(crate) use shape::names;

use oxc_ast::ast::{FormalParameters, FunctionBody};
use oxc_ast_visit::Visit;
use std::collections::HashSet;

/// Every binding identifier in the parameters or the body. A nested name
/// counts: the proof is whole-function, not a scope walk.
fn shadows(params: &FormalParameters<'_>, body: &FunctionBody<'_>) -> HashSet<String> {
    let mut names = HashSet::new();
    let mut visitor = BoundNames(&mut names);
    visitor.visit_formal_parameters(params);
    visitor.visit_function_body(body);
    names
}

struct BoundNames<'a>(&'a mut HashSet<String>);

impl<'a> Visit<'a> for BoundNames<'_> {
    fn visit_binding_identifier(&mut self, ident: &oxc_ast::ast::BindingIdentifier<'a>) {
        self.0.insert(ident.name.to_string());
    }
}
