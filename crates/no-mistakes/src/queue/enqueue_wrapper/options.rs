use super::selection::{closed_property, local_value, Selected};
use crate::queue::extract_factory::{debounce_object, dedup_id_token};
use crate::queue::extract_model::{EnqueuePriority, JobOptionEvidence, LocalCheck};
use crate::queue::extract_visitor::QueueVisitor;
use oxc_ast::ast::Expression;

pub(super) fn evidence<'a>(
    visitor: &QueueVisitor<'a>,
    layers: &[Selected<'a>],
) -> JobOptionEvidence {
    let mut evidence = JobOptionEvidence::default();
    for layer in layers {
        match layer {
            Selected::Missing => {}
            Selected::Unknown => evidence.absorb(&JobOptionEvidence::open()),
            Selected::Value(expr) => match local_value(visitor, expr) {
                Some(expr) => evidence.absorb(&crate::queue::job_options::read_evidence(
                    expr,
                    &visitor.object_bindings,
                )),
                None => evidence.absorb(&JobOptionEvidence::open()),
            },
        }
    }
    evidence
}

pub(super) fn priority<'a>(visitor: &QueueVisitor<'a>, layers: &[Selected<'a>]) -> EnqueuePriority {
    let mut priority = EnqueuePriority::Unprioritized;
    for layer in layers {
        priority = match layer {
            Selected::Missing => priority,
            Selected::Unknown => EnqueuePriority::Dynamic,
            Selected::Value(expr) => local_value(visitor, expr)
                .map(|expr| {
                    crate::queue::extract_priority::from_options_with_default(
                        expr,
                        &visitor.facts.const_numbers,
                        priority,
                    )
                })
                .unwrap_or(EnqueuePriority::Dynamic),
        };
    }
    priority
}

pub(super) fn effective_property<'a>(
    visitor: &QueueVisitor<'a>,
    layers: &[Selected<'a>],
    key: &str,
) -> Option<&'a Expression<'a>> {
    for layer in layers.iter().rev() {
        let expr = match layer {
            Selected::Missing => continue,
            Selected::Unknown => return None,
            Selected::Value(expr) => *expr,
        };
        let Expression::ObjectExpression(object) = local_value(visitor, expr)? else {
            return None;
        };
        if let Some(value) = closed_property(object, key)? {
            return Some(crate::codebase::ts_source::unwrap_ts_wrappers(value));
        }
    }
    None
}

pub(super) fn dedup<'a>(
    visitor: &QueueVisitor<'a>,
    layers: &[Selected<'a>],
    line: usize,
) -> (bool, Vec<LocalCheck>) {
    let Some(value) = effective_property(visitor, layers, "deduplication") else {
        return (false, Vec::new());
    };
    let closed = closed_dedup(value);
    let checks = (closed && debounce_object(value))
        .then(|| LocalCheck {
            line,
            kind: "debounce-ttl".to_string(),
            job: None,
            message: "deduplication ttl is ignored when mode is debounce".to_string(),
        })
        .into_iter()
        .collect();
    (closed && dedup_id_token(visitor, value), checks)
}

fn closed_dedup<'a>(value: &'a Expression<'a>) -> bool {
    match value {
        Expression::ObjectExpression(object) => closed_property(object, "id").is_some(),
        _ => true,
    }
}
