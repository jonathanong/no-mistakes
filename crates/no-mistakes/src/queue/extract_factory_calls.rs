use crate::queue::extract_model::LocalCheck;
use crate::queue::extract_visitor::QueueVisitor;
use crate::queue::source::line_number;
use oxc_ast::ast::{CallExpression, Expression};
use oxc_span::GetSpan;

#[path = "extract_factory_enqueue.rs"]
mod enqueue;

pub(crate) fn note_factory_call(visitor: &mut QueueVisitor<'_>, call: &CallExpression<'_>) {
    let Expression::Identifier(id) = &call.callee else {
        return;
    };
    let local = id.name.as_str();
    if let Some(&(queue_index, job_index)) = visitor.enqueue_factories.get(local) {
        enqueue::note_enqueue(visitor, call, queue_index, job_index);
    }
    if let Some(&(name_index, processor_index)) = visitor.worker_factories.get(local) {
        enqueue::note_worker(visitor, call, name_index, processor_index);
    }
}

pub(crate) fn note_wait(visitor: &mut QueueVisitor<'_>, call: &CallExpression<'_>) {
    visitor.facts.checks.push(LocalCheck {
        line: line_number(visitor.source, call.span.start),
        kind: "enqueue-and-wait".to_string(),
        job: None,
        message: "waiting on a job result holds a worker or request slot across queues".to_string(),
    });
}
