use super::actions::{find_action_in_stmt, find_guarded_action, for_of_names, ActionScope};
use super::expr::{
    check_state_expr, is_early_exit, match_get_job, match_listing, match_state_call, pattern_names,
};
use super::extract::{record, FnEnv};
use super::listing::{check_expr_for_actions, listing_binding};
use super::{JobStatePattern, JobStateSite};
use crate::queue::extract_visitor::QueueVisitor;
use oxc_ast::ast::{
    Declaration, ForOfStatement, IfStatement, Statement, SwitchStatement, TryStatement,
    VariableDeclaration,
};
use oxc_span::GetSpan;
use std::collections::HashSet;

struct StmtCtx<'a, 'b> {
    visitor: &'a QueueVisitor<'b>,
    env: &'a mut FnEnv,
    sites: &'a mut Vec<JobStateSite>,
    seen: &'a mut HashSet<(usize, JobStatePattern)>,
}

pub(super) fn analyze_statements(
    visitor: &QueueVisitor<'_>,
    stmts: &[Statement<'_>],
    env: &mut FnEnv,
    sites: &mut Vec<JobStateSite>,
    seen: &mut HashSet<(usize, JobStatePattern)>,
) {
    let mut ctx = StmtCtx {
        visitor,
        env,
        sites,
        seen,
    };
    ctx.walk_slice(stmts);
}

impl StmtCtx<'_, '_> {
    fn walk_slice(&mut self, stmts: &[Statement<'_>]) {
        for (index, stmt) in stmts.iter().enumerate() {
            self.walk_stmt(stmt, &stmts[index + 1..]);
        }
    }

    fn walk_stmt(&mut self, stmt: &Statement<'_>, trailing: &[Statement<'_>]) {
        match stmt {
            Statement::VariableDeclaration(decl) => self.record_decl(decl),
            Statement::ExportDeclaration(export) => self.record_export(&export.declaration),
            Statement::IfStatement(if_stmt) => self.handle_if(if_stmt, trailing),
            Statement::SwitchStatement(sw) => self.check_switch(sw),
            Statement::ForOfStatement(for_of) => self.handle_for_of(for_of),
            Statement::BlockStatement(block) => self.walk_slice(&block.body),
            Statement::TryStatement(try_stmt) => self.handle_try(try_stmt),
            Statement::ExpressionStatement(e) => {
                check_expr_for_actions(self.visitor, &e.expression, self.env, self.sites, self.seen)
            }
            Statement::ReturnStatement(r) => self.handle_return(r.argument.as_ref()),
            _ => {}
        }
    }

    fn record_export(&mut self, decl: &Declaration<'_>) {
        if let Declaration::VariableDeclaration(var_decl) = decl {
            self.record_decl(var_decl);
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
        let Some(binding) = check_state_expr(self.visitor, &if_stmt.test, self.env) else {
            return;
        };
        let action = find_guarded_action(self.visitor, &if_stmt.consequent, self.env)
            .or_else(|| {
                if_stmt
                    .alternate
                    .as_ref()
                    .and_then(|alt| find_guarded_action(self.visitor, alt, self.env))
            })
            .or_else(|| {
                (if_stmt.alternate.is_none() && is_early_exit(&if_stmt.consequent)).then(|| {
                    trailing
                        .iter()
                        .find_map(|rest| find_guarded_action(self.visitor, rest, self.env))
                })?
            });
        if let Some(action_binding) = action {
            record(
                self.visitor,
                if_stmt.span.start,
                binding,
                action_binding,
                JobStatePattern::GetJobStateGuard,
                self.sites,
                self.seen,
            );
        }
    }

    fn check_switch(&mut self, sw: &SwitchStatement<'_>) {
        let Some(binding) = check_state_expr(self.visitor, &sw.discriminant, self.env) else {
            return;
        };
        let action = sw.cases.iter().find_map(|case| {
            case.consequent
                .iter()
                .find_map(|s| find_guarded_action(self.visitor, s, self.env))
        });
        if let Some(action) = action {
            record(
                self.visitor,
                sw.span.start,
                binding,
                action,
                JobStatePattern::GetJobStateGuard,
                self.sites,
                self.seen,
            );
        }
    }

    fn handle_for_of(&mut self, for_of: &ForOfStatement<'_>) {
        if let Some(binding) = listing_binding(self.visitor, &for_of.right, self.env) {
            let loop_vars = for_of_names(&for_of.left);
            let scope = ActionScope::Listing {
                names: &loop_vars,
                start: for_of.body.span().start,
            };
            if let Some(action) = find_action_in_stmt(self.visitor, &for_of.body, scope) {
                record(
                    self.visitor,
                    for_of.span.start,
                    binding,
                    action,
                    JobStatePattern::ListingFeedsAction,
                    self.sites,
                    self.seen,
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

    fn handle_return(&mut self, arg: Option<&oxc_ast::ast::Expression<'_>>) {
        if let Some(arg) = arg {
            check_expr_for_actions(self.visitor, arg, self.env, self.sites, self.seen);
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
            check_expr_for_actions(self.visitor, init, self.env, self.sites, self.seen);
            let names = pattern_names(&declarator.id);
            let Some(name) = names.first() else {
                continue;
            };
            if let Some(binding) = match_get_job(self.visitor, init) {
                self.env.job_vars.insert(name.clone(), binding);
            } else if let Some(binding) = match_state_call(self.visitor, init, &self.env.job_vars) {
                self.env.state_vars.insert(name.clone(), binding);
            } else if let Some(binding) = match_listing(self.visitor, init) {
                self.env.listing_vars.insert(name.clone(), binding);
            }
        }
    }
}
