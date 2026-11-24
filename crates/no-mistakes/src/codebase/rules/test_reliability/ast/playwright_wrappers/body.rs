use super::super::bindings::Bindings;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Expression, FormalParameters, FunctionBody, Statement};
use oxc_ast::AstKind;
use oxc_semantic::SymbolId;

pub(super) enum WrapperBody<'ast> {
    Expr(&'ast Expression<'ast>),
    Block(&'ast FunctionBody<'ast>),
}

impl<'ast> Bindings<'_, 'ast> {
    pub(super) fn function_def(
        &self,
        symbol: SymbolId,
    ) -> Option<(&'ast FormalParameters<'ast>, WrapperBody<'ast>)> {
        let scope = self.semantic.scoping();
        match self.semantic.nodes().kind(scope.symbol_declaration(symbol)) {
            AstKind::Function(func) => {
                Some((&func.params, WrapperBody::Block(func.body.as_deref()?)))
            }
            AstKind::VariableDeclarator(decl)
                if scope.symbol_flags(symbol).is_const_variable()
                    && decl.id.is_binding_identifier() =>
            {
                match unwrap_ts_wrappers(decl.init.as_ref()?) {
                    Expression::ArrowFunctionExpression(arrow) => {
                        let body = arrow
                            .get_expression()
                            .map(WrapperBody::Expr)
                            .or_else(|| arrow.get_function_body().map(WrapperBody::Block))?;
                        Some((&arrow.params, body))
                    }
                    Expression::FunctionExpression(func) => {
                        Some((&func.params, WrapperBody::Block(func.body.as_deref()?)))
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

pub(super) fn collect_returns<'ast>(
    stmts: &'ast [Statement<'ast>],
    out: &mut Vec<&'ast Expression<'ast>>,
) -> bool {
    for stmt in stmts {
        match stmt {
            Statement::ReturnStatement(ret) => {
                let Some(arg) = ret.argument.as_ref() else {
                    return false;
                };
                out.push(arg);
            }
            Statement::BlockStatement(block) if !collect_returns(&block.body, out) => return false,
            Statement::IfStatement(if_stmt)
                if !collect_returns(std::slice::from_ref(&if_stmt.consequent), out)
                    || if_stmt
                        .alternate
                        .as_ref()
                        .is_some_and(|alt| !collect_returns(std::slice::from_ref(alt), out)) =>
            {
                return false;
            }
            _ => {}
        }
    }
    true
}
