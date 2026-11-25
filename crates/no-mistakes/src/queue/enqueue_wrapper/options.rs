use super::selection::{closed_property, local_value, Selected};
use crate::queue::extract_debounce::{
    check_dedup_value, DebounceTtlVerdict, DEBOUNCE_TTL_KIND, DEBOUNCE_TTL_MESSAGE,
};
use crate::queue::extract_factory::dedup_id_token;
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
) -> (bool, Vec<LocalCheck>, bool) {
    let Some(value) = effective_property(visitor, layers, "deduplication") else {
        return (false, Vec::new(), false);
    };
    let closed = closed_dedup(value);
    let (checks, abstained) = match check_dedup_value(visitor, value) {
        DebounceTtlVerdict::Clean => (Vec::new(), false),
        DebounceTtlVerdict::Finding => (
            vec![LocalCheck {
                line,
                kind: DEBOUNCE_TTL_KIND.to_string(),
                job: None,
                message: DEBOUNCE_TTL_MESSAGE.to_string(),
            }],
            false,
        ),
        DebounceTtlVerdict::Abstain => (Vec::new(), true),
    };
    (closed && dedup_id_token(visitor, value), checks, abstained)
}

fn closed_dedup<'a>(value: &'a Expression<'a>) -> bool {
    match value {
        Expression::ObjectExpression(object) => closed_property(object, "id").is_some(),
        _ => true,
    }
}
