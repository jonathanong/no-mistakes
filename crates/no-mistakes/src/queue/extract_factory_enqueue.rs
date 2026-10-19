use super::super::identifier_arg;
use crate::queue::extract_helpers::{
    collect_jobs_from_args, literal_or_import, processor_specifier,
};
use crate::queue::extract_model::{ProducerSite, WorkerSite};
use crate::queue::extract_visitor::QueueVisitor;
use crate::queue::source::line_number;
use oxc_ast::ast::{Argument, CallExpression, Expression};
use oxc_span::GetSpan;

pub(super) fn note_enqueue(
    visitor: &mut QueueVisitor<'_>,
    call: &CallExpression<'_>,
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
    let recorded = visitor.facts.producers.len();
    visitor.facts.producers.push(ProducerSite {
        file: visitor.path.to_path_buf(),
        line: line_number(visitor.source, call.span.start),
        binding,
        job: job.clone(),
        job_import,
        raw_job,
        worker_index: visitor.worker_stack.last().copied(),
        self_dedup: false,
    });
    if let Some(index) = options {
        if let Some(expr) = call.arguments.get(index).and_then(Argument::as_expression) {
            super::super::dedup::note_dedup_argument(visitor, call, expr, job, recorded);
        }
    }
}

pub(super) fn note_worker(
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
        processor_specifier: processor_binding(visitor, call, processor_index),
        processor_file: None,
        wildcard,
    });
    if let Some(arg) = call.arguments.get(processor_index) {
        let span = arg.span();
        let index = visitor.facts.workers.len() - 1;
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
    let arg = call.arguments.get(processor_index)?;
    if let Some(name) = identifier_arg(arg) {
        if let Some((specifier, _)) = visitor.facts.string_imports.get(&name) {
            return Some(specifier.clone());
        }
        if let Some(specifier) = visitor.namespace_imports.get(&name) {
            return Some(specifier.clone());
        }
    }
    processor_specifier(&call.arguments, visitor.source, &visitor.namespace_imports)
}
