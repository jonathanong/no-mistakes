use super::super::extract_model::JobOptionEvidence;
use super::super::extract_visitor::QueueVisitor;
use super::site::{member_expr, member_string, push_site};
use super::walk::read_member;
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
    let queue_name = member_string(object, "queueName");
    let job = member_string(object, "name");
    if object
        .properties
        .iter()
        .any(|property| matches!(property, ObjectPropertyKind::SpreadProperty(_)))
    {
        push_site(
            visitor,
            object.span().start,
            None,
            queue_name,
            job,
            JobOptionEvidence::open(),
        );
        return;
    }
    push_site(
        visitor,
        object.span().start,
        None,
        queue_name.clone(),
        job,
        read_member(object, "opts").unwrap_or_default(),
    );
    let Some(children) = member_expr(object, "children") else {
        return;
    };
    note_children(visitor, children, queue_name);
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
