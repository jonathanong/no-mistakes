use super::super::{Expr, Stmt};
use super::{expr, switch, Scope};
use oxc_ast::ast::{BindingPattern, Statement, VariableDeclarationKind};
use oxc_span::GetSpan;

/// A block's statements. `None` when a declaration reuses a name in scope,
/// which the model's flat environment cannot represent faithfully.
pub(super) fn block(scope: &mut Scope<'_, '_>, statements: &[Statement<'_>]) -> Option<Vec<Stmt>> {
    let mark = scope.declared.len();
    let mut lowered = Vec::with_capacity(statements.len());
    for statement in statements {
        lowered.extend(one(scope, statement)?);
    }
    scope.declared.truncate(mark);
    Some(lowered)
}

fn one(scope: &mut Scope<'_, '_>, statement: &Statement<'_>) -> Option<Vec<Stmt>> {
    Some(match statement {
        Statement::EmptyStatement(_) => Vec::new(),
        Statement::VariableDeclaration(declaration) => declare(scope, declaration)?,
        Statement::ExpressionStatement(statement) => {
            vec![Stmt::Expr(expr::lower(scope, &statement.expression))]
        }
        Statement::IfStatement(statement) => {
            let test = expr::lower(scope, &statement.test);
            let then = body(scope, &statement.consequent)?;
            let otherwise = match &statement.alternate {
                Some(alternate) => body(scope, alternate)?,
                None => Vec::new(),
            };
            vec![Stmt::If(test, then, otherwise)]
        }
        Statement::BlockStatement(inner) => {
            vec![Stmt::If(
                Expr::Bool(true),
                block(scope, &inner.body)?,
                Vec::new(),
            )]
        }
        Statement::ReturnStatement(statement) => {
            let mark = scope.mark(
                statement
                    .argument
                    .as_ref()
                    .map_or(statement.span, |e| e.span()),
            );
            let argument = statement.argument.as_ref().map(|e| expr::lower(scope, e));
            vec![Stmt::Return(argument, mark)]
        }
        Statement::ThrowStatement(_) => vec![Stmt::Throw],
        Statement::SwitchStatement(statement) => vec![switch::lower(scope, statement)],
        // Hoisted: a call can reach it before (or without) this statement
        // running, so the whole body must fail rather than skip the name.
        Statement::FunctionDeclaration(_) => {
            scope.hoisted = true;
            vec![Stmt::Opaque]
        }
        Statement::ForOfStatement(statement) => vec![for_of(scope, statement)?],
        _ => vec![Stmt::Opaque],
    })
}

/// `for (const x of iterable)` over a single plain binding.
fn for_of(scope: &mut Scope<'_, '_>, statement: &oxc_ast::ast::ForOfStatement<'_>) -> Option<Stmt> {
    use oxc_ast::ast::ForStatementLeft;
    let ForStatementLeft::VariableDeclaration(declaration) = &statement.left else {
        return Some(Stmt::Opaque);
    };
    let [declarator] = declaration.declarations.as_slice() else {
        return Some(Stmt::Opaque);
    };
    let BindingPattern::BindingIdentifier(ident) = &declarator.id else {
        return Some(Stmt::Opaque);
    };
    if statement.r#await || declaration.kind == VariableDeclarationKind::Var {
        return Some(Stmt::Opaque);
    }
    let name = ident.name.to_string();
    if scope.declared.contains(&name) {
        return None;
    }
    let iterable = expr::lower(scope, &statement.right);
    scope.declared.push(name.clone());
    let lowered = body(scope, &statement.body);
    scope.declared.pop();
    Some(Stmt::ForOf(name, iterable, lowered?))
}

fn body(scope: &mut Scope<'_, '_>, statement: &Statement<'_>) -> Option<Vec<Stmt>> {
    match statement {
        Statement::BlockStatement(inner) => block(scope, &inner.body),
        other => {
            let mark = scope.declared.len();
            let lowered = one(scope, other);
            scope.declared.truncate(mark);
            lowered
        }
    }
}

fn declare(
    scope: &mut Scope<'_, '_>,
    declaration: &oxc_ast::ast::VariableDeclaration<'_>,
) -> Option<Vec<Stmt>> {
    if declaration.kind == VariableDeclarationKind::Var {
        return Some(vec![Stmt::Opaque]);
    }
    let mut lowered = Vec::with_capacity(declaration.declarations.len());
    for declarator in &declaration.declarations {
        let (BindingPattern::BindingIdentifier(ident), Some(init)) =
            (&declarator.id, &declarator.init)
        else {
            return Some(vec![Stmt::Opaque]);
        };
        let name = ident.name.to_string();
        if scope.declared.contains(&name) {
            return None;
        }
        let init = expr::lower(scope, init);
        scope.declared.push(name.clone());
        lowered.push(Stmt::Let(name, init));
    }
    Some(lowered)
}

/// Whether a statement mentions SQL text: a template, literal append or return.
pub(super) fn contributes_sql(statement: &Stmt) -> bool {
    match statement {
        Stmt::Let(_, init) | Stmt::Expr(init) => expr::contributes_sql(init),
        Stmt::If(test, then, otherwise) => {
            expr::contributes_sql(test) || then.iter().chain(otherwise).any(contributes_sql)
        }
        Stmt::Return(argument, _) => argument.as_ref().is_some_and(expr::contributes_sql),
        Stmt::ForOf(_, iterable, body) => {
            expr::contributes_sql(iterable) || body.iter().any(contributes_sql)
        }
        Stmt::Switch(discriminant, arms, default) => {
            expr::contributes_sql(discriminant)
                || arms.iter().any(|arm| expr::contributes_sql(&arm.value))
                || default.as_ref().is_some_and(expr::contributes_sql)
        }
        Stmt::Throw | Stmt::Opaque => false,
    }
}
