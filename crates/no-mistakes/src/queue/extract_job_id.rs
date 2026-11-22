use crate::queue::extract_helpers::static_string;
use crate::queue::extract_visitor::QueueVisitor;
use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, Expression, IdentifierReference, VariableDeclarator,
};
use oxc_syntax::symbol::SymbolId;

#[path = "extract_job_id_message.rs"]
mod message;
#[path = "extract_job_id_reader.rs"]
mod reader;
use reader::{Evidence, Reader};

const KIND: &str = "invalid-job-id";

pub(super) fn note_const(visitor: &mut QueueVisitor<'_>, decl: &VariableDeclarator<'_>) {
    let (Some(scoping), BindingPattern::BindingIdentifier(id), Some(init)) =
        (visitor.scoping, &decl.id, &decl.init)
    else {
        return;
    };
    if let Some(symbol) = id.symbol_id.get() {
        if scoping.symbol_flags(symbol).is_const_variable() {
            if let Some(value) = static_string(init.get_inner_expression()) {
                visitor.job_id_consts.insert(symbol, value);
            }
        }
    }
}

pub(super) fn note_flow_binding(visitor: &mut QueueVisitor<'_>, pattern: &BindingPattern<'_>) {
    if let Some(symbol) = stable_binding(visitor, pattern) {
        visitor.job_id_flows.insert(symbol);
    }
}

pub(super) fn note_queue_binding(visitor: &mut QueueVisitor<'_>, pattern: &BindingPattern<'_>) {
    if let Some(symbol) = stable_binding(visitor, pattern) {
        visitor.job_id_queues.insert(symbol);
    }
}

fn stable_binding(visitor: &QueueVisitor<'_>, pattern: &BindingPattern<'_>) -> Option<SymbolId> {
    let BindingPattern::BindingIdentifier(id) = pattern else {
        return None;
    };
    let symbol = id.symbol_id.get()?;
    (!visitor.scoping?.symbol_is_mutated(symbol)).then_some(symbol)
}

fn referenced_symbol(visitor: &QueueVisitor<'_>, id: &IdentifierReference<'_>) -> Option<SymbolId> {
    visitor
        .scoping?
        .get_reference(id.reference_id.get()?)
        .symbol_id()
}

fn imported(visitor: &QueueVisitor<'_>, symbol: SymbolId) -> bool {
    visitor.scoping.is_some_and(|scoping| {
        let flags = scoping.symbol_flags(symbol);
        flags.is_import() && !flags.is_type_import()
    })
}

pub(super) fn queue_identifier(visitor: &QueueVisitor<'_>, id: &IdentifierReference<'_>) -> bool {
    referenced_symbol(visitor, id)
        .is_some_and(|symbol| visitor.job_id_queues.contains(&symbol) || imported(visitor, symbol))
}

enum Receiver {
    Queue {
        binding: String,
        symbol: SymbolId,
        namespace: bool,
    },
    Flow(SymbolId),
}

pub(super) struct PendingCheck {
    receiver: Receiver,
    evidence: Vec<Evidence>,
}

fn queue_receiver(visitor: &QueueVisitor<'_>, expr: &Expression<'_>) -> Option<Receiver> {
    let (binding, id, namespace) = match expr.get_inner_expression() {
        Expression::Identifier(id) => (id.name.to_string(), &**id, false),
        Expression::StaticMemberExpression(member) => {
            let Expression::Identifier(id) = &member.object else {
                return None;
            };
            (visitor.imported_member_binding(expr)?, &**id, true)
        }
        _ => return None,
    };
    Some(Receiver::Queue {
        binding,
        symbol: referenced_symbol(visitor, id)?,
        namespace,
    })
}

pub(super) fn note_factory_options(
    visitor: &mut QueueVisitor<'_>,
    receiver: &Expression<'_>,
    job: Option<String>,
    options: &Expression<'_>,
) {
    let Some(receiver) = queue_receiver(visitor, receiver) else {
        return;
    };
    let mut reader = Reader::new(visitor);
    reader.options(job, options);
    let evidence = reader.evidence;
    publish(visitor, receiver, evidence);
}

pub(super) fn note_call(visitor: &mut QueueVisitor<'_>, call: &CallExpression<'_>) {
    let Expression::StaticMemberExpression(member) = &call.callee else {
        return;
    };
    let method = member.property.name.as_str();
    if !matches!(method, "add" | "addBulk" | "upsertJobScheduler") {
        return;
    }
    let Some(
        receiver @ Receiver::Queue {
            symbol, namespace, ..
        },
    ) = queue_receiver(visitor, &member.object)
    else {
        return;
    };
    if method == "add" && !namespace {
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
                .and_then(|arg| visitor.literal_arg(arg));
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
            Receiver::Queue {
                binding,
                symbol,
                namespace,
            } if imported(visitor, symbol)
                || (!namespace && visitor.job_id_queues.contains(&symbol)) =>
            {
                Some(binding)
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
