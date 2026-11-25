use super::extract_job_id::referenced_symbol;
use super::extract_model::wrapper::{Name, Queue, Target, WrapperSite};
use super::extract_model::JobOptionSite;
use super::extract_visitor::QueueVisitor;
use super::source::line_number;
use crate::config::v2::schema::EnqueueFactory;
use oxc_ast::ast::{CallExpression, Expression};

mod factories;
mod mutations;
mod options;
pub(in crate::queue) use mutations::Mutations;
mod resolve;
mod selection;
mod targets;

pub(crate) use factories::needs_scoping;
pub(in crate::queue) use factories::{collect_exports, factories, Factories};
pub(in crate::queue) use resolve::resolve;
use selection::{const_value, Selected};

pub(in crate::queue) fn note_call<'a>(
    visitor: &mut QueueVisitor<'a>,
    call: &'a CallExpression<'a>,
) {
    if visitor.wrapper_factories.is_empty() {
        return;
    }
    let (factory, role) = if let Some(role) = role_for(visitor, call) {
        if role.options_on_returned_call.is_some() {
            return;
        }
        (call, role)
    } else {
        if !matches!(
            call.callee.get_inner_expression(),
            Expression::Identifier(_)
        ) {
            return;
        }
        let Some(Expression::CallExpression(factory)) = const_value(visitor, &call.callee) else {
            return;
        };
        let Some(role) = role_for(visitor, factory) else {
            return;
        };
        if role.options_on_returned_call.is_none() {
            return;
        }
        (factory.as_ref(), role)
    };
    let (target, raw_job) = targets::targets(visitor, factory, role);
    let layers = [
        Selected::select(visitor, factory, role.defaults.as_ref()),
        Selected::select(visitor, factory, role.options.as_ref()),
        role.options_on_returned_call
            .map_or(Selected::Missing, |index| Selected::argument(call, index)),
    ];
    let line = line_number(visitor.source, call.span.start);
    let options = options::evidence(visitor, &layers);
    let priority = options::priority(visitor, &layers);
    let (dedup, checks, dedup_abstained) = options::dedup(visitor, &layers, line);
    if dedup_abstained {
        visitor
            .facts
            .check_abstentions
            .push((line, super::extract_debounce::DEBOUNCE_TTL_ABSTENTION));
    }
    let wrapper = visitor.facts.enqueue_wrappers.len();
    let job_id = options::effective_property(visitor, &layers, "jobId");
    visitor.facts.enqueue_wrappers.push(WrapperSite {
        target,
        options: JobOptionSite {
            line,
            binding: None,
            queue_name: None,
            queue_file: None,
            declaration: None,
            ambiguous_defaults: false,
            job: None,
            evidence: options,
        },
        checks,
        worker_index: visitor.worker_stack.last().copied(),
        raw_job,
        priority,
        dedup: dedup || job_id.is_some_and(|expr| visitor.tracks_job_token(expr)),
        current_case: visitor.current_case.clone(),
    });
    if let Some(job_id) = job_id {
        super::extract_job_id::note_wrapper_options(visitor, wrapper, job_id);
    }
}

fn role_for<'a>(
    visitor: &QueueVisitor<'a>,
    call: &CallExpression<'_>,
) -> Option<&'a EnqueueFactory> {
    let Expression::Identifier(id) = call.callee.get_inner_expression() else {
        return None;
    };
    visitor
        .wrapper_factories
        .get(&referenced_symbol(visitor, id)?)
        .copied()
}
