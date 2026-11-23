use super::selection::{const_value, imported_local, local_value, Selected};
use super::{Name, Queue, Target};
use crate::config::v2::schema::{EnqueueFactory, EnqueuePropertySelector, EnqueueSelector};
use crate::queue::extract_helpers::static_string;
use crate::queue::extract_visitor::QueueVisitor;
use oxc_ast::ast::{CallExpression, Expression};
use oxc_span::GetSpan;

pub(super) fn targets<'a>(
    visitor: &QueueVisitor<'a>,
    call: &'a CallExpression<'a>,
    role: &EnqueueFactory,
) -> (Target, String) {
    let queue = queue(visitor, call, role);
    let named = name_at(visitor, call, &role.job);
    let raw_job = named
        .as_ref()
        .map(|(_, raw)| raw.clone())
        .unwrap_or_else(|| {
            Selected::select(visitor, call, Some(&role.job))
                .value()
                .map(|expr| crate::ast::span_text(visitor.source, expr.span()).to_owned())
                .unwrap_or_else(|| crate::ast::span_text(visitor.source, call.span).to_owned())
        });
    (
        Target {
            queue,
            job: named.map(|(name, _)| name),
        },
        raw_job,
    )
}

fn queue<'a>(
    visitor: &QueueVisitor<'a>,
    call: &'a CallExpression<'a>,
    role: &EnqueueFactory,
) -> Queue {
    if let Some(name) = &role.queue_name {
        return Queue::Name(Name::Literal(name.clone()));
    }
    let Some(selector) = &role.queue else {
        return Queue::Unknown;
    };
    if let Some((name, _)) = name_at(visitor, call, selector) {
        return Queue::Name(name);
    }
    let EnqueueSelector::Positional(index) = selector else {
        return Queue::Unknown;
    };
    let Some(expr @ Expression::Identifier(id)) = Selected::argument(call, *index).value() else {
        return Queue::Unknown;
    };
    if const_value(visitor, expr).is_none() {
        return Queue::Unknown;
    }
    Queue::Binding(visitor.producer_binding(id.name.as_str(), id.span.start))
}

fn name_at<'a>(
    visitor: &QueueVisitor<'a>,
    call: &'a CallExpression<'a>,
    selector: &EnqueueSelector,
) -> Option<(Name, String)> {
    if let Some(expr) = Selected::select(visitor, call, Some(selector)).value() {
        let raw = crate::ast::span_text(visitor.source, expr.span()).to_string();
        if let Some(value) = local_value(visitor, expr).and_then(static_string) {
            return Some((Name::Literal(value), raw));
        }
        if let Some(local) = imported_local(visitor, expr) {
            return Some((
                Name::Import {
                    local,
                    property: None,
                },
                raw,
            ));
        }
        return None;
    }
    let EnqueueSelector::Property(EnqueuePropertySelector { arg, property }) = selector else {
        return None;
    };
    let local = imported_local(visitor, Selected::argument(call, *arg).value()?)?;
    let raw = format!("{local}.{property}");
    Some((
        Name::Import {
            local,
            property: Some(property.clone()),
        },
        raw,
    ))
}
