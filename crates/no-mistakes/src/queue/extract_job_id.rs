use crate::queue::extract_model::JobIdQueue;
use crate::queue::extract_visitor::QueueVisitor;
use oxc_ast::ast::{Argument, CallExpression, Expression};
use oxc_syntax::symbol::SymbolId;

#[path = "extract_job_id_message.rs"]
mod message;
#[path = "extract_job_id_reader.rs"]
mod reader;
use reader::{Evidence, JobLabel, Reader};

const KIND: &str = "invalid-job-id";

#[path = "extract_job_id_scope.rs"]
mod scope;
use scope::imported;
pub(super) use scope::referenced_symbol;
pub(super) use scope::{
    is_imported_callee, note_const, note_flow_binding, note_queue_binding, queue_identifier,
};

enum Receiver {
    Queue {
        binding: JobIdQueue,
        symbol: SymbolId,
    },
    Flow(SymbolId),
    Wrapper(usize),
}

pub(super) struct PendingCheck {
    receiver: Receiver,
    evidence: Vec<Evidence>,
}

fn queue_receiver(visitor: &QueueVisitor<'_>, expr: &Expression<'_>) -> Option<Receiver> {
    let (binding, symbol) = match expr.get_inner_expression() {
        Expression::Identifier(id) => {
            let symbol = referenced_symbol(visitor, id)?;
            (
                JobIdQueue::Named(visitor.producer_binding(id.name.as_str(), id.span.start)),
                symbol,
            )
        }
        Expression::StaticMemberExpression(member) => {
            let Expression::Identifier(id) = &member.object else {
                return None;
            };
            let source = visitor.namespace_imports.get(id.name.as_str())?.clone();
            let exported = member.property.name.to_string();
            (
                JobIdQueue::Namespace { source, exported },
                referenced_symbol(visitor, id)?,
            )
        }
        _ => return None,
    };
    Some(Receiver::Queue { binding, symbol })
}

pub(super) fn note_factory_options(
    visitor: &mut QueueVisitor<'_>,
    receiver: &Expression<'_>,
    job: Option<&Expression<'_>>,
    options: &Expression<'_>,
) {
    let Some(receiver) = queue_receiver(visitor, receiver) else {
        return;
    };
    let mut reader = Reader::new(visitor);
    reader.options(job.and_then(|expr| JobLabel::read(visitor, expr)), options);
    let evidence = reader.evidence;
    publish(visitor, receiver, evidence);
}

pub(super) fn note_wrapper_options(
    visitor: &mut QueueVisitor<'_>,
    wrapper: usize,
    options: &Expression<'_>,
) {
    let mut reader = Reader::new(visitor);
    reader.job_id(None, options);
    let evidence = reader.evidence;
    publish(visitor, Receiver::Wrapper(wrapper), evidence);
}

pub(super) fn note_call(visitor: &mut QueueVisitor<'_>, call: &CallExpression<'_>) {
    let Expression::StaticMemberExpression(member) = &call.callee else {
        return;
    };
    let method = member.property.name.as_str();
    if !matches!(method, "add" | "addBulk" | "upsertJobScheduler") {
        return;
    }
    let Some(receiver @ Receiver::Queue { symbol, .. }) = queue_receiver(visitor, &member.object)
    else {
        return;
    };
    if method == "add"
        && matches!(
            receiver,
            Receiver::Queue {
                binding: JobIdQueue::Named(_),
                ..
            }
        )
    {
        let mut reader = Reader::new(visitor);
        reader.node_arg(call.arguments.first());
        let evidence = reader.evidence;
        publish(visitor, Receiver::Flow(symbol), evidence);
    }
    let mut reader = Reader::new(visitor);
    match method {
        "add" => {
            let job = call
                .arguments
                .first()
                .and_then(Argument::as_expression)
                .and_then(|expr| JobLabel::read(visitor, expr));
            if let Some(expr) = call.arguments.get(2).and_then(Argument::as_expression) {
                reader.options(job, expr);
            }
        }
        "addBulk" => reader.bulk(call.arguments.first()),
        _ => reader.node_arg(call.arguments.get(2)),
    }
    let evidence = reader.evidence;
    publish(visitor, receiver, evidence);
}

fn publish(visitor: &mut QueueVisitor<'_>, receiver: Receiver, evidence: Vec<Evidence>) {
    if !evidence.is_empty() {
        visitor
            .job_id_pending
            .push(PendingCheck { receiver, evidence });
    }
}

pub(super) fn finish(visitor: &mut QueueVisitor<'_>) {
    for pending in std::mem::take(&mut visitor.job_id_pending) {
        let binding = match pending.receiver {
            Receiver::Queue { binding, symbol }
                if imported(visitor, symbol)
                    || (matches!(binding, JobIdQueue::Named(_))
                        && visitor.job_id_queues.contains(&symbol)) =>
            {
                Some(binding)
            }
            Receiver::Wrapper(index) => {
                let checks = pending
                    .evidence
                    .into_iter()
                    .filter_map(|item| item.finding(&visitor.job_id_consts));
                visitor.facts.enqueue_wrappers[index].checks.extend(checks);
                continue;
            }
            Receiver::Flow(symbol) if visitor.job_id_flows.contains(&symbol) => None,
            _ => continue,
        };
        for evidence in pending.evidence {
            if let Some(check) = evidence.finding(&visitor.job_id_consts) {
                if let Some(binding) = &binding {
                    visitor.facts.job_id_sites.push((binding.clone(), check));
                } else {
                    visitor.facts.checks.push(check);
                }
            }
        }
    }
}
