use super::super::extract_model::JobOptionEvidence;
use super::super::extract_visitor::QueueVisitor;
use super::site::{member_string, push_site};
use super::walk::{read, read_member};
use oxc_ast::ast::{
    Argument, ArrayExpressionElement, CallExpression, Expression, ObjectExpression,
    ObjectPropertyKind,
};
use oxc_span::GetSpan;

pub(super) fn note_scheduler(
    visitor: &mut QueueVisitor<'_>,
    call: &CallExpression<'_>,
    binding: Option<String>,
) {
    let (job, evidence) = scheduler_template(call);
    push_site(visitor, call.span().start, binding, None, job, evidence);
}

fn scheduler_template(call: &CallExpression<'_>) -> (Option<String>, JobOptionEvidence) {
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
            read_member(object, "opts").unwrap_or_default(),
        ),
        Expression::Identifier(id) => (
            None,
            JobOptionEvidence {
                member: Some("opts".to_string()),
                spreads: vec![id.name.to_string()],
                ..JobOptionEvidence::default()
            },
        ),
        _ => (None, JobOptionEvidence::open()),
    }
}

pub(super) fn note_flow(visitor: &mut QueueVisitor<'_>, call: &CallExpression<'_>) {
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
    note_flow_node(visitor, object);
}

fn note_flow_node(visitor: &mut QueueVisitor<'_>, object: &ObjectExpression<'_>) {
    let Some(evidence) = flow_opts(object) else {
        push_site(
            visitor,
            object.span().start,
            None,
            member_string(object, "queueName"),
            member_string(object, "name"),
            JobOptionEvidence::open(),
        );
        return;
    };
    let queue_name = folded_string(object, "queueName");
    let job = folded_string(object, "name");
    let known_spreads = evidence
        .spreads
        .iter()
        .all(|name| visitor.facts.const_objects.contains_key(name));
    push_site(
        visitor,
        object.span().start,
        None,
        queue_name.clone(),
        job,
        evidence,
    );
    if known_spreads {
        if let Some(children) = folded_expr(object, "children") {
            note_children(visitor, children, queue_name);
        }
    }
}

fn flow_opts(object: &ObjectExpression<'_>) -> Option<JobOptionEvidence> {
    let mut evidence = JobOptionEvidence::default();
    let mut pending = Vec::new();
    for property in &object.properties {
        match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                match crate::ast::property_key_name(&property.key).as_deref() {
                    Some("opts") => {
                        evidence = read(&property.value);
                        pending.clear();
                    }
                    Some(_) => {}
                    None => return None,
                }
            }
            ObjectPropertyKind::SpreadProperty(spread) => {
                match spread.argument.get_inner_expression() {
                    Expression::ObjectExpression(inner) => {
                        if let Some(member) = read_member(inner, "opts") {
                            evidence = member;
                            pending.clear();
                        }
                    }
                    Expression::Identifier(id) => pending.push(id.name.to_string()),
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

fn folded_string(object: &ObjectExpression<'_>, key: &str) -> Option<String> {
    folded_expr(object, key).and_then(super::super::extract_helpers::static_string)
}

fn folded_expr<'a>(object: &'a ObjectExpression<'a>, key: &str) -> Option<&'a Expression<'a>> {
    let mut found = None;
    for property in &object.properties {
        match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                if crate::ast::property_key_name(&property.key).as_deref() == Some(key) {
                    found = Some(&property.value);
                }
            }
            ObjectPropertyKind::SpreadProperty(spread) => {
                if let Expression::ObjectExpression(inner) = spread.argument.get_inner_expression()
                {
                    if let Some(value) = folded_expr(inner, key) {
                        found = Some(value);
                    }
                }
            }
        }
    }
    found
}

fn note_children(
    visitor: &mut QueueVisitor<'_>,
    expr: &Expression<'_>,
    queue_name: Option<String>,
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
        match element {
            ArrayExpressionElement::ObjectExpression(object) => note_flow_node(visitor, object),
            _ => push_site(
                visitor,
                element.span().start,
                None,
                queue_name.clone(),
                None,
                JobOptionEvidence::open(),
            ),
        }
    }
}
