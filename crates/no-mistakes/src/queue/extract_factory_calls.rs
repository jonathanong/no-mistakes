use super::identifier_arg;
use crate::queue::extract_helpers::collect_jobs_from_args;
use crate::queue::extract_model::{LocalCheck, ProducerSite, WorkerSite};
use crate::queue::extract_visitor::QueueVisitor;
use crate::queue::source::line_number;
use oxc_ast::ast::{CallExpression, Expression};
use oxc_span::GetSpan;

pub(crate) fn note_factory_call(visitor: &mut QueueVisitor<'_>, call: &CallExpression<'_>) {
    let Expression::Identifier(id) = &call.callee else {
        return;
    };
    let local = id.name.as_str();
    if let Some(&(queue_index, job_index)) = visitor.enqueue_factories.get(local) {
        note_enqueue(visitor, call, queue_index, job_index);
    }
    if let Some(&(name_index, processor_index)) = visitor.worker_factories.get(local) {
        note_worker(visitor, call, name_index, processor_index);
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

fn note_enqueue(
    visitor: &mut QueueVisitor<'_>,
    call: &CallExpression<'_>,
    queue_index: usize,
    job_index: usize,
) {
    let Some(binding) = call.arguments.get(queue_index).and_then(identifier_arg) else {
        visitor.facts.diagnostics.push((
            line_number(visitor.source, call.span.start),
            "enqueue factory queue argument is not a local binding".to_string(),
        ));
        return;
    };
    let job = call
        .arguments
        .get(job_index)
        .and_then(|arg| visitor.literal_arg(arg));
    let raw_job = call
        .arguments
        .get(job_index)
        .map(|arg| crate::ast::span_text(visitor.source, arg.span()).to_string());
    visitor.facts.producers.push(ProducerSite {
        file: visitor.path.to_path_buf(),
        line: line_number(visitor.source, call.span.start),
        binding,
        job,
        raw_job,
        worker_index: visitor.worker_stack.last().copied(),
        self_dedup: false,
    });
}

fn note_worker(
    visitor: &mut QueueVisitor<'_>,
    call: &CallExpression<'_>,
    name_index: usize,
    processor_index: usize,
) {
    let queue_name = call
        .arguments
        .get(name_index)
        .and_then(|arg| visitor.literal_arg(arg));
    let queue_name_import = if queue_name.is_none() {
        call.arguments
            .get(name_index)
            .and_then(identifier_arg)
            .filter(|name| visitor.facts.string_imports.contains_key(name))
    } else {
        None
    };
    let mut jobs = collect_jobs_from_args(&call.arguments, visitor.source);
    jobs.sort();
    jobs.dedup();
    let wildcard = jobs.is_empty();
    visitor.facts.workers.push(WorkerSite {
        file: visitor.path.to_path_buf(),
        line: line_number(visitor.source, call.span.start),
        queue_name,
        queue_name_import,
        jobs,
        processor_specifier: None,
        processor_file: None,
        wildcard,
    });
    if let Some(arg) = call.arguments.get(processor_index) {
        let span = arg.span();
        let index = visitor.facts.workers.len() - 1;
        visitor.processor_spans.push((span.start, span.end, index));
    }
}
