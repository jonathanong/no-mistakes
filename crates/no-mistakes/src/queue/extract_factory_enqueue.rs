use super::super::identifier_arg;
use crate::queue::extract_helpers::{literal_or_import, processor_specifier};
use crate::queue::extract_model::{EnqueuePriority, ProducerSite, WorkerSite};
use crate::queue::extract_visitor::QueueVisitor;
use crate::queue::source::line_number;
use oxc_ast::ast::{Argument, CallExpression, Expression};
use oxc_span::GetSpan;

pub(super) fn note_enqueue<'a>(
    visitor: &mut QueueVisitor<'a>,
    call: &CallExpression<'a>,
    queue_index: usize,
    job_index: usize,
    options: Option<usize>,
) {
    let Some(binding) = call
        .arguments
        .get(queue_index)
        .and_then(|arg| queue_binding(visitor, arg))
    else {
        visitor.facts.diagnostics.push((
            line_number(visitor.source, call.span.start),
            "enqueue factory queue argument is not a local binding".to_string(),
        ));
        return;
    };
    let job_arg = call.arguments.get(job_index);
    let (job, job_import) = job_arg
        .and_then(Argument::as_expression)
        .map(|expr| literal_or_import(expr, &visitor.const_strings, &visitor.facts.string_imports))
        .unwrap_or((None, None));
    let raw_job = job_arg.map(|arg| crate::ast::span_text(visitor.source, arg.span()).to_string());
    if crate::queue::extract_job_id::is_imported_callee(visitor, &call.callee) {
        note_job_id_options(visitor, call, queue_index, job_arg, options);
    }
    let recorded = visitor.facts.producers.len();
    let priority = match options {
        Some(index) => crate::queue::extract_priority::from_call_argument(
            &call.arguments,
            index,
            &visitor.facts.const_numbers,
        ),
        None => EnqueuePriority::Unprioritized,
    };
    let producer_binding = visitor.producer_binding(&binding, call.span.start);
    visitor.facts.producers.push(ProducerSite {
        file: visitor.path.to_path_buf(),
        line: line_number(visitor.source, call.span.start),
        binding: producer_binding,
        job: job.clone(),
        job_import,
        raw_job,
        worker_index: visitor.worker_stack.last().copied(),
        self_dedup: false,
        priority,
    });
    crate::queue::job_options::note_argument_options(visitor, &binding, &job, call, options);
    if let Some(index) = options {
        if let Some(expr) = call.arguments.get(index).and_then(Argument::as_expression) {
            super::super::dedup::note_dedup_argument(visitor, call, expr, job, recorded);
        }
    }
}

fn note_job_id_options(
    visitor: &mut QueueVisitor<'_>,
    call: &CallExpression<'_>,
    queue_index: usize,
    job: Option<&Argument<'_>>,
    options: Option<usize>,
) {
    if let Some(expr) = options
        .and_then(|index| call.arguments.get(index))
        .and_then(Argument::as_expression)
    {
        if let Some(receiver) = call
            .arguments
            .get(queue_index)
            .and_then(Argument::as_expression)
        {
            crate::queue::extract_job_id::note_factory_options(
                visitor,
                receiver,
                job.and_then(Argument::as_expression),
                expr,
            );
        }
    }
}

pub(super) fn note_worker(
    visitor: &mut QueueVisitor<'_>,
    call: &CallExpression<'_>,
    name_index: usize,
    processor_index: usize,
) {
    let (queue_name, queue_name_import) = call
        .arguments
        .get(name_index)
        .map(|arg| visitor.resolve_queue_name_arg(arg))
        .unwrap_or((None, None));
    visitor.facts.workers.push(WorkerSite {
        file: visitor.path.to_path_buf(),
        line: line_number(visitor.source, call.span.start),
        queue_name,
        queue_name_import,
        jobs: Vec::new(),
        processor_specifier: processor_binding(visitor, call, processor_index),
        processor_file: None,
        wildcard: true,
    });
    let index = visitor.facts.workers.len() - 1;
    visitor
        .worker_rate_limit
        .note_worker_span(call.span.start, index);
    if let Some(arg) = call.arguments.get(processor_index) {
        if crate::queue::extract_job_id::is_imported_callee(visitor, &call.callee) {
            visitor.note_rate_limit_processor(arg, index, true);
        }
        let span = arg.span();
        visitor.processor_spans.push((span.start, span.end, index));
    }
}

fn queue_binding(visitor: &mut QueueVisitor<'_>, arg: &Argument<'_>) -> Option<String> {
    if let Some(name) = identifier_arg(arg) {
        return Some(name);
    }
    let Expression::StaticMemberExpression(member) = arg.as_expression()? else {
        return None;
    };
    let Expression::Identifier(object) = &member.object else {
        return None;
    };
    let specifier = visitor.namespace_imports.get(object.name.as_str())?;
    let member_name = member.property.name.as_str();
    let binding = format!("{}.{member_name}", object.name);
    visitor.facts.string_imports.insert(
        binding.clone(),
        (specifier.clone(), member_name.to_string()),
    );
    Some(binding)
}

fn processor_binding(
    visitor: &QueueVisitor<'_>,
    call: &CallExpression<'_>,
    processor_index: usize,
) -> Option<String> {
    let expr = call
        .arguments
        .get(processor_index)?
        .as_expression()?
        .get_inner_expression();
    if let Expression::Identifier(id) = expr {
        if let Some(import) = visitor
            .facts
            .imports
            .iter()
            .find(|import| import.local == id.name.as_str())
        {
            return Some(import.source.clone());
        }
    }
    processor_specifier(expr, &visitor.namespace_imports)
}
