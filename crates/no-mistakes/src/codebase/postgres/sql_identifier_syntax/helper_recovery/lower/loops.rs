//! One execution of a loop, so SQL in the body is visible at low effort.
use super::{statements, Expr, Scope, Stmt};
use oxc_ast::ast::{
    BindingPattern, ForStatementInit, ForStatementLeft, Statement, VariableDeclaration,
};
use oxc_ast_visit::{walk, Visit};

pub(super) fn lower(scope: &mut Scope<'_>, node: &Statement<'_>) -> Vec<Stmt> {
    match node {
        Statement::ForStatement(statement) => c_for(scope, statement),
        Statement::ForInStatement(statement) => {
            for_each(scope, &statement.left, &statement.right, &statement.body)
        }
        Statement::ForOfStatement(statement) => {
            for_each(scope, &statement.left, &statement.right, &statement.body)
        }
        Statement::WhileStatement(statement) => {
            while_loop(scope, &statement.test, &statement.body, true)
        }
        Statement::DoWhileStatement(statement) => {
            while_loop(scope, &statement.test, &statement.body, false)
        }
        _ => vec![Stmt::Opaque],
    }
}

fn c_for(scope: &mut Scope<'_>, statement: &oxc_ast::ast::ForStatement<'_>) -> Vec<Stmt> {
    let mut out = Vec::new();
    if let Some(init) = &statement.init {
        match init {
            ForStatementInit::VariableDeclaration(declaration) => {
                if !push_bindings(scope, declaration, &mut out) {
                    return vec![Stmt::Opaque];
                }
            }
            other => {
                let Some(expression) = other.as_expression() else {
                    return vec![Stmt::Opaque];
                };
                out.push(Stmt::Expr(effect(scope, expression)));
            }
        }
    }
    if let Some(test) = &statement.test {
        out.push(Stmt::Expr(effect(scope, test)));
    }
    let mut out = extend_body(scope, &statement.body, out);
    if let Some(update) = &statement.update {
        out.push(Stmt::Expr(effect(scope, update)));
    }
    out
}

fn for_each(
    scope: &mut Scope<'_>,
    left: &ForStatementLeft<'_>,
    right: &oxc_ast::ast::Expression<'_>,
    body: &Statement<'_>,
) -> Vec<Stmt> {
    let mut out = vec![Stmt::Expr(effect(scope, right))];
    match left {
        ForStatementLeft::VariableDeclaration(declaration) => {
            if declaration.declarations.len() != 1 {
                return vec![Stmt::Opaque];
            }
            let declarator = &declaration.declarations[0];
            let Some(name) = binding_name(scope, &declarator.id) else {
                return vec![Stmt::Opaque];
            };
            out.push(Stmt::Let(name, Expr::Num));
        }
        _ => return vec![Stmt::Opaque],
    }
    extend_body(scope, body, out)
}

fn while_loop(
    scope: &mut Scope<'_>,
    test: &oxc_ast::ast::Expression<'_>,
    body: &Statement<'_>,
    test_first: bool,
) -> Vec<Stmt> {
    let test = Stmt::Expr(effect(scope, test));
    let Some(mut body) = statements::block(scope, std::slice::from_ref(body)) else {
        return vec![Stmt::Opaque];
    };
    if test_first {
        let mut out = vec![test];
        out.append(&mut body);
        out
    } else {
        body.push(test);
        body
    }
}

fn push_bindings(
    scope: &mut Scope<'_>,
    declaration: &VariableDeclaration<'_>,
    out: &mut Vec<Stmt>,
) -> bool {
    for declarator in &declaration.declarations {
        let Some(name) = binding_name(scope, &declarator.id) else {
            return false;
        };
        let value = declarator
            .init
            .as_ref()
            .map(|init| scope.expression(init))
            .unwrap_or(Expr::Num);
        out.push(Stmt::Let(name, value));
    }
    true
}

fn extend_body(scope: &mut Scope<'_>, body: &Statement<'_>, mut out: Vec<Stmt>) -> Vec<Stmt> {
    match statements::block(scope, std::slice::from_ref(body)) {
        Some(body) => {
            out.extend(body);
            out
        }
        None => vec![Stmt::Opaque],
    }
}

fn binding_name(scope: &Scope<'_>, pattern: &BindingPattern<'_>) -> Option<String> {
    let BindingPattern::BindingIdentifier(binding) = pattern else {
        return None;
    };
    scope.ctx.bindings.name(binding.span)
}

fn effect(scope: &mut Scope<'_>, expression: &oxc_ast::ast::Expression<'_>) -> Expr {
    if carries_call_or_template(expression) {
        scope.expression(expression)
    } else {
        Expr::Num
    }
}

fn carries_call_or_template(expression: &oxc_ast::ast::Expression<'_>) -> bool {
    struct Probe(bool);
    impl<'a> Visit<'a> for Probe {
        fn visit_call_expression(&mut self, node: &oxc_ast::ast::CallExpression<'a>) {
            self.0 = true;
            walk::walk_call_expression(self, node);
        }
        fn visit_template_literal(&mut self, node: &oxc_ast::ast::TemplateLiteral<'a>) {
            self.0 = true;
            walk::walk_template_literal(self, node);
        }
        fn visit_tagged_template_expression(
            &mut self,
            node: &oxc_ast::ast::TaggedTemplateExpression<'a>,
        ) {
            self.0 = true;
            walk::walk_tagged_template_expression(self, node);
        }
    }
    let mut probe = Probe(false);
    probe.visit_expression(expression);
    probe.0
}
