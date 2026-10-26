use super::{Expr, Scope, Stmt};
use oxc_ast::ast::{BindingPattern, Statement, VariableDeclarationKind};
use oxc_span::GetSpan;

pub(super) fn block(scope: &mut Scope<'_>, statements: &[Statement<'_>]) -> Option<Vec<Stmt>> {
    let mut result = Vec::new();
    for node in statements {
        scope.work += 1;
        if scope.work > 10_000 {
            result.push(Stmt::Opaque);
            break;
        }
        match node {
            Statement::EmptyStatement(_) => {}
            Statement::VariableDeclaration(declaration)
                if declaration.kind != VariableDeclarationKind::Var =>
            {
                for declarator in &declaration.declarations {
                    let value = declarator
                        .init
                        .as_ref()
                        .map_or(Expr::Undefined, |init| scope.expression(init));
                    match &declarator.id {
                        BindingPattern::BindingIdentifier(binding) => {
                            result.push(Stmt::Let(scope.ctx.bindings.name(binding.span)?, value));
                        }
                        BindingPattern::ObjectPattern(pattern) => {
                            let slot = format!("local:{}", declarator.span.start);
                            result.push(Stmt::Let(slot.clone(), value));
                            match super::parameters::destructure(scope, pattern, &slot) {
                                Some(prologue) => result.extend(prologue),
                                None => result.push(Stmt::Opaque),
                            }
                        }
                        _ => {
                            result.push(Stmt::Expr(value));
                            result.push(Stmt::Opaque);
                        }
                    }
                }
            }
            Statement::ExpressionStatement(statement) => {
                result.push(Stmt::Expr(scope.expression(&statement.expression)))
            }
            Statement::ReturnStatement(statement) => {
                let mark = scope.mark(
                    statement
                        .argument
                        .as_ref()
                        .map_or(statement.span, GetSpan::span),
                );
                result.push(Stmt::Return(
                    statement.argument.as_ref().map(|arg| scope.expression(arg)),
                    mark,
                ));
            }
            Statement::IfStatement(statement) => {
                let test = scope.expression(&statement.test);
                let yes = one(scope, &statement.consequent)?;
                let no = statement
                    .alternate
                    .as_ref()
                    .map_or(Some(Vec::new()), |alt| one(scope, alt))?;
                result.push(Stmt::If(test, yes, no));
            }
            Statement::BlockStatement(block_node) => result.extend(block(scope, &block_node.body)?),
            // Throw arguments can execute or escape values. Closedness must inspect
            // them rather than silently treating every throw as a harmless end.
            Statement::ThrowStatement(_) => result.push(Stmt::Opaque),
            _ => result.push(Stmt::Opaque),
        }
    }
    Some(result)
}
fn one(scope: &mut Scope<'_>, statement: &Statement<'_>) -> Option<Vec<Stmt>> {
    block(scope, std::slice::from_ref(statement))
}
