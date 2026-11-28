use super::super::extract_model::JobOptionEvidence;
use super::super::extract_visitor::QueueVisitor;
use super::site::{member_string, push_site};
use super::walk::{read, read_member};
use crate::fx::FxHashSet;
use oxc_ast::ast::{Argument, CallExpression, Expression, ObjectExpression, ObjectPropertyKind};
use oxc_span::GetSpan;

#[path = "job_options_flow_members.rs"]
mod members;

pub(super) fn note_scheduler(
    visitor: &mut QueueVisitor<'_>,
    call: &CallExpression<'_>,
    binding: String,
) {
    let (job, evidence) = scheduler_template(call, &visitor.object_bindings);
    push_site(
        visitor,
        call.span().start,
        Some(binding),
        None,
        job,
        evidence,
    );
}

fn scheduler_template(
    call: &CallExpression<'_>,
    bindings: &super::super::object_bindings::ObjectBindings<'_>,
) -> (Option<String>, JobOptionEvidence) {
    let Some(arg) = call.arguments.get(2) else {
        let evidence = if call
            .arguments
            .iter()
            .any(|arg| matches!(arg, Argument::SpreadElement(_)))
        {
            JobOptionEvidence::open()
        } else {
            JobOptionEvidence::default()
        };
        return (None, evidence);
    };
    let Some(expr) = arg.as_expression() else {
        return (None, JobOptionEvidence::open());
    };
    match expr.get_inner_expression() {
        Expression::ObjectExpression(object) => (
            member_string(object, "name"),
            read_member(object, "opts", bindings).unwrap_or_default(),
        ),
        Expression::Identifier(id) => (
            None,
            JobOptionEvidence {
                member: Some("opts".to_string()),
                spreads: vec![(&**id).into()],
                ..JobOptionEvidence::default()
            },
        ),
        _ => (None, JobOptionEvidence::open()),
    }
}

pub(super) fn note_flow<'a>(visitor: &mut QueueVisitor<'a>, call: &CallExpression<'a>) {
    let Some(Expression::ObjectExpression(object)) = call
        .arguments
        .first()
        .and_then(Argument::as_expression)
        .map(Expression::get_inner_expression)
    else {
        push_site(
            visitor,
            call.span().start,
            None,
            None,
            None,
            JobOptionEvidence::open(),
        );
        return;
    };
    note_flow_node(visitor, object, &mut FxHashSet::default());
}

fn note_flow_node<'a>(
    visitor: &mut QueueVisitor<'a>,
    object: &ObjectExpression<'a>,
    active: &mut FxHashSet<u32>,
) {
    let start = object.span().start;
    if !active.insert(start) {
        return;
    }
    let members = members::read(object, &visitor.object_bindings);
    let queue_name = members.queue_name();
    let evidence = flow_opts(object, &visitor.object_bindings);
    let job = members.job();
    crate::queue::extract_debounce::check_flow_node(visitor, object, job.clone());
    push_site(
        visitor,
        start,
        None,
        queue_name.clone(),
        job,
        evidence.unwrap_or_else(JobOptionEvidence::open),
    );
    if let Some(children) = members.children() {
        note_children(visitor, children, queue_name, active);
    }
    active.remove(&start);
}

fn flow_opts(
    object: &ObjectExpression<'_>,
    bindings: &super::super::object_bindings::ObjectBindings<'_>,
) -> Option<JobOptionEvidence> {
    let mut evidence = JobOptionEvidence::default();
    let mut pending = Vec::new();
    for property in &object.properties {
        match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                match crate::ast::property_key_name(&property.key).as_deref() {
                    Some("opts") => {
                        evidence = read(&property.value, bindings);
                        pending.clear();
                    }
                    Some(_) => {}
                    None => return None,
                }
            }
            ObjectPropertyKind::SpreadProperty(spread) => {
                match spread.argument.get_inner_expression() {
                    Expression::ObjectExpression(inner) => {
                        if let Some(member) = read_member(inner, "opts", bindings) {
                            evidence = member;
                            pending.clear();
                        }
                    }
                    Expression::Identifier(id) => pending.push((&**id).into()),
                    _ => return None,
                }
            }
        }
    }
    if !pending.is_empty() {
        evidence.member = Some("opts".to_string());
        evidence.spreads = pending;
    }
    Some(evidence)
}

fn note_children<'a>(
    visitor: &mut QueueVisitor<'a>,
    expr: &Expression<'a>,
    queue_name: Option<String>,
    active: &mut FxHashSet<u32>,
) {
    let expr = expr.get_inner_expression();
    let Expression::ArrayExpression(array) = expr else {
        push_site(
            visitor,
            expr.span().start,
            None,
            queue_name,
            None,
            JobOptionEvidence::open(),
        );
        return;
    };
    for element in &array.elements {
        if let Some(Expression::ObjectExpression(object)) = element
            .as_expression()
            .map(Expression::get_inner_expression)
        {
            note_flow_node(visitor, object, active);
        } else {
            push_site(
                visitor,
                element.span().start,
                None,
                queue_name.clone(),
                None,
                JobOptionEvidence::open(),
            );
        }
    }
}
