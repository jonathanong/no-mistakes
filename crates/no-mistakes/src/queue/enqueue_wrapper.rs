use super::extract_helpers::static_string;
use super::extract_job_id::referenced_symbol;
use super::extract_model::{JobOptionSite, LocalCheck, ProducerBinding};
use super::extract_visitor::QueueVisitor;
use super::source::line_number;
use crate::config::v2::schema::{EnqueueFactory, EnqueuePropertySelector, EnqueueSelector};
use oxc_ast::ast::{CallExpression, Expression};
use oxc_span::GetSpan;

mod factories;
mod mutations;
mod options;
pub(in crate::queue) use mutations::Mutations;
mod resolve;
mod selection;

pub(crate) use factories::needs_scoping;
pub(in crate::queue) use factories::{collect_exports, factories, Factories};
pub(in crate::queue) use resolve::resolve;
use selection::{imported_local, local_value, Selected};

#[derive(Debug, Clone)]
pub(crate) struct WrapperSite {
    target: Target,
    pub options: JobOptionSite,
    pub checks: Vec<LocalCheck>,
    worker_index: Option<usize>,
    raw_job: String,
    priority: super::extract_model::EnqueuePriority,
    dedup: bool,
    current_case: Option<String>,
}

#[derive(Debug, Clone)]
struct Target {
    queue: Queue,
    job: Name,
}

#[derive(Debug, Clone)]
enum Queue {
    Name(Name),
    Binding(ProducerBinding),
}

#[derive(Debug, Clone)]
enum Name {
    Literal(String),
    Import {
        local: String,
        property: Option<String>,
    },
}

pub(in crate::queue) fn note_call<'a>(
    visitor: &mut QueueVisitor<'a>,
    call: &'a CallExpression<'a>,
) {
    if visitor.wrapper_factories.is_empty() {
        return;
    }
    let (factory, role) = if let Some(role) = role_for(visitor, call) {
        if role.options_on_returned_call.is_some() {
            return;
        }
        (call, role)
    } else {
        if !matches!(
            call.callee.get_inner_expression(),
            Expression::Identifier(_)
        ) {
            return;
        }
        let Some(Expression::CallExpression(factory)) = local_value(visitor, &call.callee) else {
            return;
        };
        let Some(role) = role_for(visitor, factory) else {
            return;
        };
        if role.options_on_returned_call.is_none() {
            return;
        }
        (factory.as_ref(), role)
    };
    let Some((target, raw_job)) = targets(visitor, factory, role) else {
        return;
    };
    let layers = [
        Selected::select(visitor, factory, role.defaults.as_ref()),
        Selected::select(visitor, factory, role.options.as_ref()),
        role.options_on_returned_call
            .map_or(Selected::Missing, |index| Selected::argument(call, index)),
    ];
    let line = line_number(visitor.source, call.span.start);
    let options = options::evidence(visitor, &layers);
    let priority = options::priority(visitor, &layers);
    let (dedup, checks) = options::dedup(visitor, &layers, line);
    let wrapper = visitor.facts.enqueue_wrappers.len();
    let job_id = options::effective_property(visitor, &layers, "jobId");
    visitor.facts.enqueue_wrappers.push(WrapperSite {
        target,
        options: JobOptionSite {
            line,
            binding: None,
            queue_name: None,
            queue_file: None,
            job: None,
            evidence: options,
        },
        checks,
        worker_index: visitor.worker_stack.last().copied(),
        raw_job,
        priority,
        dedup: dedup || job_id.is_some_and(|expr| visitor.tracks_job_token(expr)),
        current_case: visitor.current_case.clone(),
    });
    if let Some(job_id) = job_id {
        super::extract_job_id::note_wrapper_options(visitor, wrapper, job_id);
    }
}

fn role_for<'a>(
    visitor: &QueueVisitor<'a>,
    call: &CallExpression<'_>,
) -> Option<&'a EnqueueFactory> {
    let Expression::Identifier(id) = call.callee.get_inner_expression() else {
        return None;
    };
    visitor
        .wrapper_factories
        .get(&referenced_symbol(visitor, id)?)
        .copied()
}

fn targets<'a>(
    visitor: &QueueVisitor<'a>,
    call: &'a CallExpression<'a>,
    role: &EnqueueFactory,
) -> Option<(Target, String)> {
    let queue = if let Some(name) = &role.queue_name {
        Queue::Name(Name::Literal(name.clone()))
    } else if let Some((name, _)) = name_at(visitor, call, role.queue.as_ref()?) {
        match (&role.queue, &name) {
            (Some(EnqueueSelector::Positional(_)), Name::Import { local, .. }) => {
                Queue::Binding(visitor.producer_binding(local, call.span.start))
            }
            _ => Queue::Name(name),
        }
    } else {
        let Some(EnqueueSelector::Positional(index)) = &role.queue else {
            return None;
        };
        let Expression::Identifier(id) = Selected::argument(call, *index).value()? else {
            return None;
        };
        Queue::Binding(visitor.producer_binding(id.name.as_str(), id.span.start))
    };
    let (job, raw_job) = name_at(visitor, call, &role.job)?;
    Some((Target { queue, job }, raw_job))
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
