//! Collect dispatch facts in the existing queue AST pass, never from source text.
use super::extract_helpers::literal_expr;
use super::extract_visitor::QueueVisitor;
use oxc_ast::ast::{
    BinaryExpression, ComputedMemberExpression, Expression, IdentifierReference,
    StaticMemberExpression, SwitchStatement,
};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::operator::BinaryOperator;

pub(super) fn visit_binary<'a>(visitor: &mut QueueVisitor<'a>, expr: &BinaryExpression<'a>) {
    if matches!(
        expr.operator,
        BinaryOperator::Equality | BinaryOperator::StrictEquality
    ) {
        for (name, value) in [(&expr.left, &expr.right), (&expr.right, &expr.left)] {
            if visitor.tracks_job_name(name) {
                if let Some(job) = literal_expr(value, &visitor.const_strings) {
                    note_job(visitor, job);
                    // The recognized name operand must not also count as dynamic dispatch.
                    visitor.visit_expression(value);
                    return;
                }
            }
        }
    }
    walk::walk_binary_expression(visitor, expr);
}

pub(super) fn note_switch(visitor: &mut QueueVisitor<'_>, stmt: &SwitchStatement<'_>) {
    if !visitor.tracks_job_name(&stmt.discriminant) {
        return;
    }
    for test in stmt.cases.iter().filter_map(|case| case.test.as_ref()) {
        if let Some(job) = literal_expr(test, &visitor.const_strings) {
            note_job(visitor, job);
        } else {
            note_dynamic(visitor);
        }
    }
}

pub(super) fn visit_member<'a>(
    visitor: &mut QueueVisitor<'a>,
    member: &StaticMemberExpression<'a>,
) {
    if member.property.name == "name" {
        if let Expression::Identifier(object) = &member.object {
            if visitor
                .job_frames
                .iter()
                .any(|frame| frame.object.as_deref() == Some(object.name.as_str()))
            {
                note_dynamic(visitor);
            }
        }
    }
    walk::walk_static_member_expression(visitor, member);
}

pub(super) fn visit_computed_member<'a>(
    visitor: &mut QueueVisitor<'a>,
    member: &ComputedMemberExpression<'a>,
) {
    if let Expression::Identifier(object) = &member.object {
        if literal_expr(&member.expression, &visitor.const_strings).is_none_or(|key| key == "name")
            && visitor
                .job_frames
                .iter()
                .any(|frame| frame.object.as_deref() == Some(object.name.as_str()))
        {
            note_dynamic(visitor);
        }
    }
    walk::walk_computed_member_expression(visitor, member);
}

pub(super) fn visit_identifier(visitor: &mut QueueVisitor<'_>, id: &IdentifierReference<'_>) {
    if visitor.job_frames.iter().any(|frame| {
        frame
            .name_idents
            .iter()
            .any(|name| name == id.name.as_str())
    }) {
        note_dynamic(visitor);
    }
}

fn note_job(visitor: &mut QueueVisitor<'_>, job: String) {
    visitor.facts.referenced_jobs.push(job.clone());
    if let Some(&worker) = visitor.worker_stack.last() {
        visitor.dispatch_jobs.entry(worker).or_default().push(job);
    }
}

fn note_dynamic(visitor: &mut QueueVisitor<'_>) {
    visitor.facts.referenced_jobs_dynamic = true;
    if let Some(&worker) = visitor.worker_stack.last() {
        visitor.dispatch_dynamic_workers.insert(worker);
    }
}

pub(super) fn finish(visitor: &mut QueueVisitor<'_>) {
    // File-level imports cannot select between dispatching callables soundly.
    if visitor.dispatch_scopes.len() > 1 {
        visitor.facts.referenced_jobs.clear();
        visitor.facts.referenced_jobs_dynamic = true;
    }
    visitor.facts.referenced_jobs.sort();
    visitor.facts.referenced_jobs.dedup();
    for (index, worker) in visitor.facts.workers.iter_mut().enumerate() {
        worker.jobs = visitor.dispatch_jobs.remove(&index).unwrap_or_default();
        worker.jobs.sort();
        worker.jobs.dedup();
        worker.wildcard =
            worker.jobs.is_empty() || visitor.dispatch_dynamic_workers.contains(&index);
    }
}
