use crate::ast;
use crate::integration_tests::types::CallTarget;
use oxc_ast::ast::CallExpression;
use oxc_ast_visit::{walk, Visit};

#[derive(Default)]
pub(super) struct CallCollector {
    pub(super) calls: Vec<CallTarget>,
    function_depth: usize,
}

impl<'a> Visit<'a> for CallCollector {
    fn visit_arrow_function_expression(
        &mut self,
        function: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    ) {
        if self.function_depth > 0 {
            return;
        }
        self.function_depth += 1;
        walk::walk_arrow_function_expression(self, function);
        self.function_depth -= 1;
    }

    fn visit_function(
        &mut self,
        function: &oxc_ast::ast::Function<'a>,
        flags: oxc_syntax::scope::ScopeFlags,
    ) {
        if self.function_depth > 0 {
            return;
        }
        self.function_depth += 1;
        walk::walk_function(self, function, flags);
        self.function_depth -= 1;
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Some(path) = ast::expression_path(&call.callee) {
            if path.len() == 1 {
                self.calls.push(CallTarget::Local(path[0].clone()));
            } else if path.len() == 2 {
                self.calls.push(CallTarget::Namespace {
                    namespace: path[0].clone(),
                    member: path[1].clone(),
                });
            } else if let Some(local) = path.first() {
                self.calls.push(CallTarget::Imported {
                    local: local.clone(),
                });
            }
        }
        walk::walk_call_expression(self, call);
    }
}
