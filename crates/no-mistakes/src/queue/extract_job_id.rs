use crate::ast::{js_literal, property_key_name, template_quasi_text};
use crate::queue::extract_helpers::literal_expr;
use crate::queue::extract_model::{FileFacts, LocalCheck};
use crate::queue::source::line_number;
use oxc_ast::ast::{
    Argument, ArrayExpressionElement, CallExpression, Expression, ObjectExpression,
    ObjectPropertyKind,
};
use std::collections::HashMap;

#[path = "extract_job_id_message.rs"]
mod message;

const KIND: &str = "invalid-job-id";

pub(super) fn note_options_job_id(
    source: &str,
    consts: &HashMap<String, String>,
    facts: &mut FileFacts,
    job: Option<String>,
    options: &Expression<'_>,
) {
    let Expression::ObjectExpression(object) = options.get_inner_expression() else {
        return;
    };
    note_direct_job_id(source, consts, facts, job, object);
}

pub(super) fn note_bulk_item(
    source: &str,
    consts: &HashMap<String, String>,
    facts: &mut FileFacts,
    object: &ObjectExpression<'_>,
) {
    note_node(source, consts, facts, object, false);
}

pub(super) fn note_flow_node(
    source: &str,
    consts: &HashMap<String, String>,
    facts: &mut FileFacts,
    expr: &Expression<'_>,
) {
    let Expression::ObjectExpression(object) = expr.get_inner_expression() else {
        return;
    };
    note_node(source, consts, facts, object, true);
}

pub(super) fn note_scheduler(
    source: &str,
    consts: &HashMap<String, String>,
    facts: &mut FileFacts,
    call: &CallExpression<'_>,
) {
    let Expression::StaticMemberExpression(member) = &call.callee else {
        return;
    };
    if member.property.name != "upsertJobScheduler" {
        return;
    }
    let Some(template) = call.arguments.get(2).and_then(Argument::as_expression) else {
        return;
    };
    note_flow_node(source, consts, facts, template);
}

fn note_node(
    source: &str,
    consts: &HashMap<String, String>,
    facts: &mut FileFacts,
    object: &ObjectExpression<'_>,
    walk_children: bool,
) {
    let job = property_literal(object, consts, "name");
    note_direct_job_id(source, consts, facts, job.clone(), object);
    if let Some(opts) = property_expr(object, "opts") {
        note_options_job_id(source, consts, facts, job, opts);
    }
    if !walk_children {
        return;
    }
    let Some(children) = property_expr(object, "children") else {
        return;
    };
    let Expression::ArrayExpression(array) = children.get_inner_expression() else {
        return;
    };
    for element in &array.elements {
        if let ArrayExpressionElement::ObjectExpression(child) = element {
            note_node(source, consts, facts, child, true);
        }
    }
}

fn note_direct_job_id(
    source: &str,
    consts: &HashMap<String, String>,
    facts: &mut FileFacts,
    job: Option<String>,
    object: &ObjectExpression<'_>,
) {
    for (key, value) in properties(object) {
        if property_key_name(key).as_deref() == Some("jobId") {
            note_job_id_expr(source, consts, facts, job.clone(), value);
        }
    }
}

fn properties<'a>(
    object: &'a ObjectExpression<'a>,
) -> impl Iterator<Item = (&'a oxc_ast::ast::PropertyKey<'a>, &'a Expression<'a>)> {
    object.properties.iter().filter_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        Some((&property.key, &property.value))
    })
}

fn property_expr<'a>(object: &'a ObjectExpression<'a>, name: &str) -> Option<&'a Expression<'a>> {
    properties(object)
        .find_map(|(key, value)| (property_key_name(key).as_deref() == Some(name)).then_some(value))
}

fn property_literal(
    object: &ObjectExpression<'_>,
    consts: &HashMap<String, String>,
    name: &str,
) -> Option<String> {
    property_expr(object, name).and_then(|expr| literal_expr(expr, consts))
}

fn note_job_id_expr(
    source: &str,
    consts: &HashMap<String, String>,
    facts: &mut FileFacts,
    job: Option<String>,
    expr: &Expression<'_>,
) {
    match expr.get_inner_expression() {
        Expression::StringLiteral(literal) => {
            push_invalid(
                source,
                facts,
                job,
                literal.span.start,
                js_literal(literal).as_ref(),
            );
        }
        Expression::TemplateLiteral(template) => {
            for quasi in &template.quasis {
                push_invalid(
                    source,
                    facts,
                    job.clone(),
                    quasi.span.start,
                    template_quasi_text(quasi).as_ref(),
                );
            }
        }
        Expression::Identifier(id) => {
            if let Some(value) = consts.get(id.name.as_str()) {
                push_invalid(source, facts, job, id.span.start, value);
            }
        }
        _ => {}
    }
}

fn push_invalid(source: &str, facts: &mut FileFacts, job: Option<String>, start: u32, text: &str) {
    let Some(message) = message::invalid_message(text) else {
        return;
    };
    facts.checks.push(LocalCheck {
        line: line_number(source, start),
        kind: KIND.to_string(),
        job,
        message,
    });
}
