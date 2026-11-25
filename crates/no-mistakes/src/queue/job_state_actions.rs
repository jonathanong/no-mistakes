use super::expr::{call_member, unwrap_expr};
use super::extract::ExtractCtx;
pub(super) use super::item::{Action, ActionScope};
use crate::queue::object_bindings::collect_names;
use oxc_ast::ast::{
    ArrowFunctionExpression, AssignmentExpression, AssignmentOperator, AssignmentTarget,
    BindingPattern, BlockStatement, CallExpression, CatchClause, Expression, ForInStatement,
    ForOfStatement, ForStatement, FormalParameters, Function, Statement, SwitchStatement,
    VariableDeclarator,
};
use oxc_ast_visit::{walk, Visit};

pub(super) fn find_in_expr(
    ctx: &mut ExtractCtx<'_, '_>,
    expr: &Expression<'_>,
    scope: ActionScope<'_>,
) -> Option<Action> {
    let mut finder = ActionFinder {
        ctx,
        scope,
        found: None,
    };
    finder.visit_expression(expr);
    finder.found
}

pub(super) fn find_in_stmts(
    ctx: &mut ExtractCtx<'_, '_>,
    stmts: &[Statement<'_>],
    scope: ActionScope<'_>,
) -> Option<Action> {
    let mut finder = ActionFinder {
        ctx,
        scope,
        found: None,
    };
    for stmt in stmts {
        finder.visit_statement(stmt);
        if finder.proven() {
            break;
        }
    }
    finder.found
}

struct ActionFinder<'a, 'b, 'c> {
    ctx: &'a mut ExtractCtx<'b, 'c>,
    scope: ActionScope<'a>,
    found: Option<Action>,
}

impl ActionFinder<'_, '_, '_> {
    fn proven(&self) -> bool {
        matches!(self.found, Some(Action::Retry | Action::Enqueue(_)))
    }

    fn with_scope(&mut self, walk_inner: impl FnOnce(&mut Self)) {
        let saved = match &self.scope {
            ActionScope::StateGuard(names) => Some(names.clone()),
            ActionScope::Listing(_) => None,
        };
        walk_inner(self);
        if let (ActionScope::StateGuard(names), Some(saved)) = (&mut self.scope, saved) {
            *names = saved;
        }
    }

    fn shadow_pattern(&mut self, pattern: &BindingPattern<'_>) {
        if let ActionScope::StateGuard(job_names) = &mut self.scope {
            let mut names = Vec::new();
            collect_names(pattern, &mut names);
            job_names.retain(|name| !names.contains(&name.as_str()));
        }
    }

    fn shadow_params(&mut self, params: &FormalParameters<'_>) {
        for param in &params.items {
            self.shadow_pattern(&param.pattern);
        }
        if let Some(rest) = &params.rest {
            self.shadow_pattern(&rest.rest.argument);
        }
    }

    fn update_binding(&mut self, name: &str, init: &Expression<'_>, can_alias: bool) {
        if let ActionScope::StateGuard(job_names) = &mut self.scope {
            let aliases = can_alias
                && matches!(
                    unwrap_expr(init),
                    Expression::Identifier(src) if job_names.iter().any(|n| n == src.name.as_str())
                );
            job_names.retain(|n| n != name);
            if aliases {
                job_names.push(name.to_string());
            }
        }
    }
}

impl<'a> Visit<'a> for ActionFinder<'_, '_, '_> {
    fn visit_function(&mut self, _func: &Function<'a>, _flags: oxc_syntax::scope::ScopeFlags) {}
    fn visit_arrow_function_expression(&mut self, _arrow: &ArrowFunctionExpression<'a>) {}
    fn visit_block_statement(&mut self, block: &BlockStatement<'a>) {
        self.with_scope(|this| walk::walk_block_statement(this, block));
    }
    fn visit_for_statement(&mut self, stmt: &ForStatement<'a>) {
        self.with_scope(|this| walk::walk_for_statement(this, stmt));
    }
    fn visit_for_of_statement(&mut self, stmt: &ForOfStatement<'a>) {
        self.with_scope(|this| walk::walk_for_of_statement(this, stmt));
    }
    fn visit_for_in_statement(&mut self, stmt: &ForInStatement<'a>) {
        self.with_scope(|this| walk::walk_for_in_statement(this, stmt));
    }
    fn visit_switch_statement(&mut self, stmt: &SwitchStatement<'a>) {
        self.with_scope(|this| walk::walk_switch_statement(this, stmt));
    }
    fn visit_catch_clause(&mut self, clause: &CatchClause<'a>) {
        self.with_scope(|this| {
            if let Some(param) = &clause.param {
                this.shadow_pattern(&param.pattern);
            }
            walk::walk_block_statement(this, &clause.body);
        });
    }

    fn visit_variable_declarator(&mut self, decl: &VariableDeclarator<'a>) {
        if let Some(init) = &decl.init {
            self.visit_expression(init);
            if let BindingPattern::BindingIdentifier(id) = &decl.id {
                self.update_binding(id.name.as_str(), init, true);
                return;
            }
        }
        self.shadow_pattern(&decl.id);
    }

    fn visit_assignment_expression(&mut self, assign: &AssignmentExpression<'a>) {
        self.visit_expression(&assign.right);
        if let AssignmentTarget::AssignmentTargetIdentifier(id) = &assign.left {
            self.update_binding(
                id.name.as_str(),
                &assign.right,
                assign.operator == AssignmentOperator::Assign,
            );
        }
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if self.proven() {
            return;
        }
        match call.callee.get_inner_expression() {
            Expression::ArrowFunctionExpression(arrow) => {
                self.with_scope(|this| {
                    this.shadow_params(&arrow.params);
                    this.visit_arrow_function_body(&arrow.body);
                });
                return;
            }
            Expression::FunctionExpression(func) => {
                if let Some(body) = &func.body {
                    self.with_scope(|this| {
                        this.shadow_params(&func.params);
                        this.visit_function_body(body);
                    });
                }
                return;
            }
            _ => {}
        }
        if let Some((object, method)) = call_member(call) {
            let job_action = method == "retry"
                || (method == "remove" && matches!(self.scope, ActionScope::StateGuard(_)));
            if job_action {
                if let Expression::Identifier(id) = unwrap_expr(object) {
                    if self.scope.matches(self.ctx.visitor, id) {
                        self.found = Some(Action::Retry);
                        return;
                    }
                }
            } else if matches!(method, "add" | "addBulk") {
                if let Some(binding) = self.ctx.queue_receiver(object, call.span.start) {
                    if let Some(action) = self.scope.enqueue(self.ctx.visitor, call, binding) {
                        self.found = Some(action);
                        if self.proven() {
                            return;
                        }
                    }
                }
            }
        }
        walk::walk_call_expression(self, call);
    }
}
