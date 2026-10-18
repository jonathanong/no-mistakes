use super::{resolve, ScopeVisitor};
use oxc_ast::ast;
use oxc_ast_visit::walk;
use oxc_syntax::scope::ScopeFlags;

impl<'a> ScopeVisitor<'a> {
    pub(super) fn walk_catch_clause_scope(&mut self, clause: &oxc_ast::ast::CatchClause<'a>) {
        self.push_scope();
        if let Some(param) = &clause.param {
            self.bind_param(&param.pattern, false);
        }
        walk::walk_catch_clause(self, clause);
        self.pop_scope();
    }

    pub(super) fn walk_function_scope(&mut self, function: &ast::Function<'a>, flags: ScopeFlags) {
        self.push_scope();
        self.record_params(&function.params);
        // A named function expression shadows outer helpers only in its own
        // body. LocalFunctions does not resolve through this self-reference,
        // so record a shadow marker in the scope just pushed.
        if function.r#type == ast::FunctionType::FunctionExpression {
            if let Some(id) = &function.id {
                self.bind_self_name(id.name.as_str());
            }
        }
        self.enter_function();
        walk::walk_function(self, function, flags);
        self.leave_function();
        self.pop_scope();
    }

    pub(super) fn walk_class_scope(&mut self, class: &oxc_ast::ast::Class<'a>) {
        let private_name = (class.r#type == oxc_ast::ast::ClassType::ClassExpression)
            .then_some(class.id.as_ref())
            .flatten();
        if let Some(name) = private_name {
            self.push_scope();
            self.bind_self_name(name.name.as_str());
        }
        // The private class name shadows outer bindings in heritage expressions,
        // computed keys and method bodies; it must not escape the expression.
        walk::walk_class(self, class);
        if private_name.is_some() {
            self.pop_scope();
        }
    }

    pub(super) fn walk_static_block_scope(&mut self, block: &oxc_ast::ast::StaticBlock<'a>) {
        // Class static blocks are also independent var ownership boundaries.
        self.push_scope();
        self.enter_function();
        resolve::hoist_vars(&block.body, self);
        resolve::record_statements(&block.body, self);
        walk::walk_static_block(self, block);
        self.leave_function();
        self.pop_scope();
    }
}
