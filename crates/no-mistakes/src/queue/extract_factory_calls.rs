use super::identifier_arg;
use crate::queue::extract_helpers::collect_jobs_from_args;
use crate::queue::extract_model::{LocalCheck, ProducerSite, WorkerSite};
use crate::queue::extract_visitor::QueueVisitor;
use crate::queue::source::line_number;
use oxc_ast::ast::{Argument, CallExpression, Expression, ObjectPropertyKind};
use oxc_span::GetSpan;

pub(crate) fn note_factory_call(visitor: &mut QueueVisitor<'_>, call: &CallExpression<'_>) {
    let Expression::Identifier(id) = &call.callee else {
        return;
    };
    let local = id.name.as_str();
    if let Some(&(queue_index, job_index)) = visitor.enqueue_factories.get(local) {
        note_enqueue(visitor, call, queue_index, job_index);
    }
    if let Some(&(name_index, _)) = visitor.worker_factories.get(local) {
        note_worker(visitor, call, name_index);
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

pub(crate) fn note_self_dedup(visitor: &mut QueueVisitor<'_>, call: &CallExpression<'_>) {
    if let Some(expr) = call.arguments.get(2).and_then(Argument::as_expression) {
        if debounce_with_ttl(expr) {
            visitor.facts.checks.push(LocalCheck {
                line: line_number(visitor.source, call.span.start),
                kind: "debounce-ttl".to_string(),
                job: call
                    .arguments
                    .first()
                    .and_then(|arg| visitor.literal_arg(arg)),
                message: "deduplication ttl is ignored when mode is debounce".to_string(),
            });
        }
    }
    if visitor.in_job == 0 {
        return;
    }
    let Some(job) = call
        .arguments
        .first()
        .and_then(|arg| visitor.literal_arg(arg))
    else {
        return;
    };
    if visitor
        .current_case
        .as_ref()
        .is_some_and(|case| case != &job)
    {
        return;
    }
    let Some(expr) = call.arguments.get(2).and_then(Argument::as_expression) else {
        return;
    };
    if !dedup_uses_active_job(expr) {
        return;
    }
    visitor.facts.checks.push(LocalCheck {
        line: line_number(visitor.source, call.span.start),
        kind: "self-dedup-enqueue".to_string(),
        job: Some(job),
        message: "enqueue of the active job reuses its deduplication id or job id, so the queue drops the add while that job is active".to_string(),
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
    });
}

fn note_worker(visitor: &mut QueueVisitor<'_>, call: &CallExpression<'_>, name_index: usize) {
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
}

fn debounce_with_ttl(expr: &Expression<'_>) -> bool {
    let Expression::ObjectExpression(object) = expr else {
        return false;
    };
    object.properties.iter().any(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return false;
        };
        property.key.static_name().as_deref() == Some("deduplication")
            && debounce_object(&property.value)
    })
}

fn debounce_object(expr: &Expression<'_>) -> bool {
    let Expression::ObjectExpression(object) = expr else {
        return false;
    };
    let mut debounce = false;
    let mut ttl = false;
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        match property.key.static_name().as_deref() {
            Some("mode") => {
                debounce = matches!(
                    &property.value,
                    Expression::StringLiteral(value) if value.value == "debounce"
                );
            }
            Some("ttl") => ttl = true,
            _ => {}
        }
    }
    debounce && ttl
}

fn dedup_uses_active_job(expr: &Expression<'_>) -> bool {
    let Expression::ObjectExpression(object) = expr else {
        return false;
    };
    object.properties.iter().any(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return false;
        };
        match property.key.static_name().as_deref() {
            Some("jobId") => is_job_token(&property.value),
            Some("deduplication") => dedup_id_token(&property.value),
            _ => false,
        }
    })
}

fn dedup_id_token(expr: &Expression<'_>) -> bool {
    let Expression::ObjectExpression(object) = expr else {
        return is_job_token(expr);
    };
    object.properties.iter().any(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return false;
        };
        property.key.static_name().as_deref() == Some("id") && is_job_token(&property.value)
    })
}

fn is_job_token(expr: &Expression<'_>) -> bool {
    let Expression::StaticMemberExpression(member) = expr else {
        return false;
    };
    let Expression::Identifier(object) = &member.object else {
        return false;
    };
    object.name == "job" && matches!(member.property.name.as_str(), "id" | "deduplicationId")
}
