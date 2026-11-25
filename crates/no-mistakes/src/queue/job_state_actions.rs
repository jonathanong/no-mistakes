use super::expr::{call_member, queue_receiver, unwrap_expr};
use super::extract::FnEnv;
use crate::queue::extract_model::ProducerBinding;
use crate::queue::extract_visitor::QueueVisitor;
use oxc_ast::ast::{CallExpression, Expression, Statement};
use oxc_ast_visit::{walk, Visit};

pub(super) fn find_guarded_action(
    visitor: &QueueVisitor<'_>,
    stmt: &Statement<'_>,
    env: &FnEnv,
) -> Option<Option<ProducerBinding>> {
    let names: Vec<_> = env.job_vars.keys().cloned().collect();
    find_action_in_stmt(visitor, stmt, &names, true)
}

pub(super) fn find_listing_action(
    visitor: &QueueVisitor<'_>,
    stmt: &Statement<'_>,
    loop_vars: &[String],
) -> Option<Option<ProducerBinding>> {
    find_action_in_stmt(visitor, stmt, loop_vars, false)
}

pub(super) fn find_action_in_expr(
    visitor: &QueueVisitor<'_>,
    expr: &Expression<'_>,
    job_names: &[String],
    allow_remove: bool,
) -> Option<Option<ProducerBinding>> {
    let mut finder = ActionFinder {
        visitor,
        job_names,
        allow_remove,
        found: None,
    };
    finder.visit_expression(expr);
    finder.found
}

fn find_action_in_stmt(
    visitor: &QueueVisitor<'_>,
    stmt: &Statement<'_>,
    job_names: &[String],
    allow_remove: bool,
) -> Option<Option<ProducerBinding>> {
    let mut finder = ActionFinder {
        visitor,
        job_names,
        allow_remove,
        found: None,
    };
    finder.visit_statement(stmt);
    finder.found
}

struct ActionFinder<'c, 'b> {
    visitor: &'b QueueVisitor<'c>,
    job_names: &'b [String],
    allow_remove: bool,
    found: Option<Option<ProducerBinding>>,
}

impl<'a> Visit<'a> for ActionFinder<'_, '_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if self.found.is_some() {
            return;
        }
        if let Some((object, method)) = call_member(call) {
            let job_action = method == "retry" || (self.allow_remove && method == "remove");
            if job_action {
                if let Expression::Identifier(id) = unwrap_expr(object) {
                    if self.job_names.iter().any(|name| name == id.name.as_str()) {
                        self.found = Some(None);
                        return;
                    }
                }
            } else if matches!(method, "add" | "addBulk") {
                if let Some(binding) = queue_receiver(self.visitor, object, call.span.start) {
                    self.found = Some(Some(binding));
                    return;
                }
            }
        }
        walk::walk_call_expression(self, call);
    }
}
