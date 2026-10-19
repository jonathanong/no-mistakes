use crate::queue::extract_model::LocalCheck;
use crate::queue::extract_visitor::QueueVisitor;
use crate::queue::source::line_number;
use oxc_ast::ast::{Argument, CallExpression, Expression, ObjectPropertyKind};

pub(crate) fn note_self_dedup(
    visitor: &mut QueueVisitor<'_>,
    call: &CallExpression<'_>,
    recorded: usize,
) {
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
    for producer in &mut visitor.facts.producers[recorded..] {
        producer.self_dedup = true;
    }
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
