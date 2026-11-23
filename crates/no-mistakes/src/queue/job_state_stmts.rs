use super::actions::{find_guarded_action, find_listing_action};
use super::expr::{
    check_state_expr, is_early_exit, match_get_job, match_listing, match_state_call, pattern_names,
};
use super::extract::{record, FnEnv};
use super::listing::{check_expr_for_actions, listing_binding};
use super::{JobStatePattern, JobStateSite};
use crate::queue::extract_visitor::QueueVisitor;
use oxc_ast::ast::{Declaration, ForStatementLeft, IfStatement, Statement, VariableDeclaration};
use std::collections::HashSet;

pub(super) fn analyze_statements(
    visitor: &QueueVisitor<'_>,
    stmts: &[Statement<'_>],
    env: &mut FnEnv,
    sites: &mut Vec<JobStateSite>,
    seen: &mut HashSet<(usize, JobStatePattern)>,
) {
    for (index, stmt) in stmts.iter().enumerate() {
        match stmt {
            Statement::VariableDeclaration(decl) => record_decl(visitor, decl, env, sites, seen),
            Statement::ExportDeclaration(export) => {
                if let Declaration::VariableDeclaration(decl) = &export.declaration {
                    record_decl(visitor, decl, env, sites, seen);
                }
            }
            Statement::IfStatement(if_stmt) => {
                check_if(visitor, if_stmt, &stmts[index + 1..], env, sites, seen);
                analyze_child(visitor, &if_stmt.consequent, env, sites, seen);
                if let Some(alt) = &if_stmt.alternate {
                    analyze_child(visitor, alt, env, sites, seen);
                }
            }
            Statement::SwitchStatement(sw) => {
                if let Some(binding) = check_state_expr(visitor, &sw.discriminant, env) {
                    if let Some(action) = sw.cases.iter().find_map(|case| {
                        case.consequent
                            .iter()
                            .find_map(|s| find_guarded_action(visitor, s, env))
                    }) {
                        record(
                            visitor,
                            sw.span.start,
                            binding,
                            action,
                            JobStatePattern::GetJobStateGuard,
                            sites,
                            seen,
                        );
                    }
                }
            }
            Statement::ForOfStatement(for_of) => {
                if let Some(binding) = listing_binding(visitor, &for_of.right, env) {
                    let loop_vars = for_of_names(&for_of.left);
                    if let Some(action) = find_listing_action(visitor, &for_of.body, &loop_vars) {
                        record(
                            visitor,
                            for_of.span.start,
                            binding,
                            action,
                            JobStatePattern::ListingFeedsAction,
                            sites,
                            seen,
                        );
                    }
                }
                analyze_child(visitor, &for_of.body, env, sites, seen);
            }
            Statement::BlockStatement(block) => {
                analyze_statements(visitor, &block.body, env, sites, seen);
            }
            Statement::TryStatement(t) => {
                analyze_statements(visitor, &t.block.body, env, sites, seen);
                if let Some(h) = &t.handler {
                    analyze_statements(visitor, &h.body.body, env, sites, seen);
                }
                if let Some(f) = &t.finalizer {
                    analyze_statements(visitor, &f.body, env, sites, seen);
                }
            }
            Statement::ExpressionStatement(e) => {
                check_expr_for_actions(visitor, &e.expression, env, sites, seen);
            }
            Statement::ReturnStatement(r) => {
                if let Some(arg) = &r.argument {
                    check_expr_for_actions(visitor, arg, env, sites, seen);
                }
            }
            _ => {}
        }
    }
}

fn check_if(
    visitor: &QueueVisitor<'_>,
    if_stmt: &IfStatement<'_>,
    trailing: &[Statement<'_>],
    env: &FnEnv,
    sites: &mut Vec<JobStateSite>,
    seen: &mut HashSet<(usize, JobStatePattern)>,
) {
    let Some(binding) = check_state_expr(visitor, &if_stmt.test, env) else {
        return;
    };
    let action = find_guarded_action(visitor, &if_stmt.consequent, env)
        .or_else(|| {
            if_stmt
                .alternate
                .as_ref()
                .and_then(|alt| find_guarded_action(visitor, alt, env))
        })
        .or_else(|| {
            (if_stmt.alternate.is_none() && is_early_exit(&if_stmt.consequent)).then(|| {
                trailing
                    .iter()
                    .find_map(|rest| find_guarded_action(visitor, rest, env))
            })?
        });
    if let Some(action_binding) = action {
        record(
            visitor,
            if_stmt.span.start,
            binding,
            action_binding,
            JobStatePattern::GetJobStateGuard,
            sites,
            seen,
        );
    }
}

fn analyze_child(
    visitor: &QueueVisitor<'_>,
    stmt: &Statement<'_>,
    env: &mut FnEnv,
    sites: &mut Vec<JobStateSite>,
    seen: &mut HashSet<(usize, JobStatePattern)>,
) {
    match stmt {
        Statement::BlockStatement(block) => {
            analyze_statements(visitor, &block.body, env, sites, seen);
        }
        other => analyze_statements(visitor, std::slice::from_ref(other), env, sites, seen),
    }
}

fn record_decl(
    visitor: &QueueVisitor<'_>,
    decl: &VariableDeclaration<'_>,
    env: &mut FnEnv,
    sites: &mut Vec<JobStateSite>,
    seen: &mut HashSet<(usize, JobStatePattern)>,
) {
    for declarator in &decl.declarations {
        let Some(init) = &declarator.init else {
            continue;
        };
        check_expr_for_actions(visitor, init, env, sites, seen);
        let names = pattern_names(&declarator.id);
        let Some(name) = names.first() else {
            continue;
        };
        if let Some(binding) = match_get_job(visitor, init) {
            env.job_vars.insert(name.clone(), binding);
        } else if let Some(binding) = match_state_call(visitor, init, &env.job_vars) {
            env.state_vars.insert(name.clone(), binding);
        } else if let Some(binding) = match_listing(visitor, init) {
            env.listing_vars.insert(name.clone(), binding);
        }
    }
}

fn for_of_names(left: &ForStatementLeft<'_>) -> Vec<String> {
    match left {
        ForStatementLeft::VariableDeclaration(decl) => decl
            .declarations
            .iter()
            .flat_map(|d| pattern_names(&d.id))
            .collect(),
        _ => Vec::new(),
    }
}
