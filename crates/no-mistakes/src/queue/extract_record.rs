use crate::ast;
use crate::queue::extract::{FileFacts, ProducerSite};
use crate::queue::extract_helpers::{literal_expr, literal_or_import};
use crate::queue::source::line_number;
use oxc_ast::ast::{Argument, CallExpression, ObjectPropertyKind};
use oxc_span::GetSpan;
use std::collections::HashMap;
use std::path::Path;

pub(super) fn record_enqueue(
    binding: &str,
    method: &str,
    path: &Path,
    source: &str,
    consts: &HashMap<String, String>,
    facts: &mut FileFacts,
    call: &CallExpression<'_>,
) {
    if method == "add" {
        let (job, job_import) = call
            .arguments
            .first()
            .and_then(Argument::as_expression)
            .map(|expr| literal_or_import(expr, consts, &facts.string_imports))
            .unwrap_or((None, None));
        let priority = crate::queue::extract_priority::from_call_argument(
            &call.arguments,
            2,
            &facts.const_numbers,
        );
        facts.producers.push(ProducerSite {
            file: path.to_path_buf(),
            line: line_number(source, call.span.start),
            binding: binding.to_string(),
            raw_job: call.arguments.first().map(|arg| span(source, arg.span())),
            job,
            job_import,
            worker_index: None,
            self_dedup: false,
            priority,
        });
        return;
    }
    record_bulk(binding, path, source, consts, facts, call);
}

fn record_bulk(
    binding: &str,
    path: &Path,
    source: &str,
    consts: &HashMap<String, String>,
    facts: &mut FileFacts,
    call: &CallExpression<'_>,
) {
    let Some(argument) = call.arguments.first() else {
        return;
    };
    let Argument::ArrayExpression(array) = argument else {
        facts.opaque_priority_bindings.insert(binding.to_string());
        return;
    };
    let mut opaque = false;
    for element in &array.elements {
        if let oxc_ast::ast::ArrayExpressionElement::ObjectExpression(object) = element {
            let recorded = facts.producers.len();
            record_bulk_item(binding, path, source, consts, facts, call, object);
            if facts.producers.len() == recorded {
                opaque = true;
            }
        } else {
            opaque = true;
        }
    }
    if opaque {
        facts.opaque_priority_bindings.insert(binding.to_string());
    }
}

pub(super) fn record_flow(
    path: &Path,
    source: &str,
    consts: &HashMap<String, String>,
    facts: &mut FileFacts,
    call: &CallExpression<'_>,
) {
    let Some(Argument::ObjectExpression(object)) = call.arguments.first() else {
        return;
    };
    let mut queue = None;
    let mut job = None;
    let mut job_import = None;
    let mut raw_job = None;
    for property in &object.properties {
        if let ObjectPropertyKind::ObjectProperty(property) = property {
            match crate::ast::property_key_name(&property.key).as_deref() {
                Some("queueName") => queue = literal_expr(&property.value, consts),
                Some("name") => {
                    raw_job = Some(span(source, property.value.span()));
                    (job, job_import) =
                        literal_or_import(&property.value, consts, &facts.string_imports);
                }
                _ => {}
            }
        }
    }
    let Some(queue) = queue else {
        return;
    };
    let priority = crate::queue::extract_priority::from_job_object(object, &facts.const_numbers);
    let binding = format!("__flow__{queue}");
    facts.queue_bindings.insert(binding.clone(), queue.clone());
    facts.queue_exports.insert(binding.clone(), queue);
    facts.producers.push(ProducerSite {
        file: path.to_path_buf(),
        line: line_number(source, call.span.start),
        binding,
        raw_job,
        job,
        job_import,
        worker_index: None,
        self_dedup: false,
        priority,
    });
}

fn record_bulk_item(
    binding: &str,
    path: &Path,
    source: &str,
    consts: &HashMap<String, String>,
    facts: &mut FileFacts,
    call: &CallExpression<'_>,
    object: &oxc_ast::ast::ObjectExpression<'_>,
) {
    let priority = crate::queue::extract_priority::from_job_object(object, &facts.const_numbers);
    let Some(name) = effective_bulk_name(object) else {
        return;
    };
    let (job, job_import) = literal_or_import(name, consts, &facts.string_imports);
    facts.producers.push(ProducerSite {
        file: path.to_path_buf(),
        line: line_number(source, call.span.start),
        binding: binding.to_string(),
        raw_job: Some(span(source, name.span())),
        job,
        job_import,
        worker_index: None,
        self_dedup: false,
        priority,
    });
}

fn effective_bulk_name<'a, 'b>(
    object: &'b oxc_ast::ast::ObjectExpression<'a>,
) -> Option<&'b oxc_ast::ast::Expression<'a>> {
    for property in object.properties.iter().rev() {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        match crate::ast::property_key_name(&property.key).as_deref() {
            Some("name") => return Some(&property.value),
            Some(_) => {}
            None => return None,
        }
    }
    None
}

fn span(source: &str, span: oxc_span::Span) -> String {
    ast::span_text(source, span).trim().to_string()
}
