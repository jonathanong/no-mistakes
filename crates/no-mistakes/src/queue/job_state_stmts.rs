use super::actions::{find_in_stmts, ActionMode};
use super::expr::{
    check_state_expr, for_of_bindings, is_early_exit, match_get_job, match_state_call, unwrap_expr,
};
use super::extract::{ExtractCtx, FnEnv};
use super::listing::{check_expr_for_actions, derived_listing_binding};
use super::JobStatePattern;
use oxc_ast::ast::{
    AssignmentTarget, BindingPattern, Declaration, Expression, ForOfStatement, IfStatement,
    Statement, SwitchStatement, TryStatement, VariableDeclaration,
};

struct StmtCtx<'a, 'b, 'c> {
    ctx: &'a mut ExtractCtx<'b, 'c>,
    env: &'a mut FnEnv,
}

pub(super) fn analyze_statements(
    ctx: &mut ExtractCtx<'_, '_>,
    stmts: &[Statement<'_>],
    env: &mut FnEnv,
) {
    StmtCtx { ctx, env }.walk_slice(stmts);
}

impl StmtCtx<'_, '_, '_> {
    fn walk_slice(&mut self, stmts: &[Statement<'_>]) {
        for (index, stmt) in stmts.iter().enumerate() {
            self.walk_stmt(stmt, &stmts[index + 1..]);
        }
    }

    fn walk_stmt(&mut self, stmt: &Statement<'_>, trailing: &[Statement<'_>]) {
        match stmt {
            Statement::VariableDeclaration(decl) => self.record_decl(decl),
            Statement::ExportDeclaration(export) => {
                if let Declaration::VariableDeclaration(var_decl) = &export.declaration {
                    self.record_decl(var_decl);
                }
            }
            Statement::IfStatement(if_stmt) => self.handle_if(if_stmt, trailing),
            Statement::SwitchStatement(sw) => self.check_switch(sw),
            Statement::ForOfStatement(for_of) => self.handle_for_of(for_of),
            Statement::BlockStatement(block) => self.walk_slice(&block.body),
            Statement::TryStatement(try_stmt) => self.handle_try(try_stmt),
            Statement::ExpressionStatement(e) => {
                check_expr_for_actions(self.ctx, &e.expression, self.env);
                if let Expression::AssignmentExpression(assign) = &e.expression {
                    if let AssignmentTarget::AssignmentTargetIdentifier(id) = &assign.left {
                        self.bind_var(id.name.as_str(), &assign.right);
                    }
                }
            }
            Statement::ReturnStatement(r) => {
                if let Some(arg) = &r.argument {
                    check_expr_for_actions(self.ctx, arg, self.env);
                }
            }
            _ => {}
        }
    }

    fn handle_if(&mut self, if_stmt: &IfStatement<'_>, trailing: &[Statement<'_>]) {
        self.check_if(if_stmt, trailing);
        self.walk_child(&if_stmt.consequent);
        if let Some(alt) = &if_stmt.alternate {
            self.walk_child(alt);
        }
    }

    fn check_if(&mut self, if_stmt: &IfStatement<'_>, trailing: &[Statement<'_>]) {
        let Some(matched) = check_state_expr(self.ctx, &if_stmt.test, self.env) else {
            return;
        };
        let names = self.env.job_names_for(matched.job_id);
        let mode = ActionMode::Guarded;
        let cons = std::slice::from_ref(&if_stmt.consequent);
        let action = find_in_stmts(self.ctx, cons, &names, &names, mode)
            .or_else(|| {
                if_stmt.alternate.as_ref().and_then(|alt| {
                    find_in_stmts(self.ctx, std::slice::from_ref(alt), &names, &names, mode)
                })
            })
            .or_else(|| {
                (if_stmt.alternate.is_none() && is_early_exit(&if_stmt.consequent))
                    .then(|| find_in_stmts(self.ctx, trailing, &names, &names, mode))?
            });
        if let Some(action_binding) = action {
            self.ctx.record(
                if_stmt.span.start,
                matched.binding,
                action_binding,
                JobStatePattern::GetJobStateGuard,
            );
        }
    }

    fn check_switch(&mut self, sw: &SwitchStatement<'_>) {
        let Some(matched) = check_state_expr(self.ctx, &sw.discriminant, self.env) else {
            return;
        };
        let names = self.env.job_names_for(matched.job_id);
        let action = sw.cases.iter().find_map(|case| {
            find_in_stmts(
                self.ctx,
                &case.consequent,
                &names,
                &names,
                ActionMode::Guarded,
            )
        });
        if let Some(action) = action {
            self.ctx.record(
                sw.span.start,
                matched.binding,
                action,
                JobStatePattern::GetJobStateGuard,
            );
        }
    }

    fn handle_for_of(&mut self, for_of: &ForOfStatement<'_>) {
        if let Some(binding) = derived_listing_binding(self.ctx, &for_of.right, self.env) {
            let (job_vars, arg_vars) = for_of_bindings(&for_of.left);
            let body = std::slice::from_ref(&for_of.body);
            if let Some(action) =
                find_in_stmts(self.ctx, body, &job_vars, &arg_vars, ActionMode::Listing)
            {
                self.ctx.record(
                    for_of.span.start,
                    binding,
                    action,
                    JobStatePattern::ListingFeedsAction,
                );
            }
        }
        self.walk_child(&for_of.body);
    }

    fn handle_try(&mut self, try_stmt: &TryStatement<'_>) {
        self.walk_slice(&try_stmt.block.body);
        if let Some(handler) = &try_stmt.handler {
            self.walk_slice(&handler.body.body);
        }
        if let Some(finalizer) = &try_stmt.finalizer {
            self.walk_slice(&finalizer.body);
        }
    }

    fn walk_child(&mut self, stmt: &Statement<'_>) {
        match stmt {
            Statement::BlockStatement(block) => self.walk_slice(&block.body),
            other => self.walk_slice(std::slice::from_ref(other)),
        }
    }

    fn record_decl(&mut self, decl: &VariableDeclaration<'_>) {
        for declarator in &decl.declarations {
            let Some(init) = &declarator.init else {
                continue;
            };
            check_expr_for_actions(self.ctx, init, self.env);
            if let BindingPattern::BindingIdentifier(id) = &declarator.id {
                self.bind_var(id.name.as_str(), init);
            }
        }
    }

    fn bind_var(&mut self, name: &str, init: &Expression<'_>) {
        self.env.job_vars.remove(name);
        self.env.state_vars.remove(name);
        self.env.listing_vars.remove(name);
        let key = name.to_string();
        if let Some(binding) = match_get_job(self.ctx, init) {
            self.env.alloc_job(key, binding);
        } else if let Expression::Identifier(src) = unwrap_expr(init) {
            if let Some(job) = self.env.job_vars.get(src.name.as_str()).cloned() {
                self.env.job_vars.insert(key, job);
            } else if let Some(state) = self.env.state_vars.get(src.name.as_str()).cloned() {
                self.env.state_vars.insert(key, state);
            } else if let Some(list) = self.env.listing_vars.get(src.name.as_str()).cloned() {
                self.env.listing_vars.insert(key, list);
            }
        } else if let Some(matched) = match_state_call(self.ctx, init, &self.env.job_vars) {
            self.env.state_vars.insert(key, matched);
        } else if let Some(binding) = derived_listing_binding(self.ctx, init, self.env) {
            self.env.listing_vars.insert(key, binding);
        }
    }
}
