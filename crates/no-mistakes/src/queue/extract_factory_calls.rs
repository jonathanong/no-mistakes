use crate::config::v2::schema::QueueFactoryRoles;
use crate::fx::{fx_map, FxHashMap};
use crate::queue::extract_helpers::export_name;
use crate::queue::extract_model::LocalCheck;
use crate::queue::extract_visitor::QueueVisitor;
use crate::queue::source::line_number;
use oxc_ast::ast::{CallExpression, Expression, ImportDeclarationSpecifier, Program, Statement};
use oxc_syntax::symbol::SymbolId;

#[path = "extract_factory_enqueue.rs"]
mod enqueue;
#[path = "extract_factory_scheduler.rs"]
mod scheduler;

pub(crate) fn note_factory_call(visitor: &mut QueueVisitor<'_>, call: &CallExpression<'_>) {
    scheduler::note_scheduler(visitor, call);
    let Expression::Identifier(id) = &call.callee else {
        return;
    };
    let local = id.name.as_str();
    if let Some(&(queue_index, job_index, options)) = visitor.enqueue_factories.get(local) {
        enqueue::note_enqueue(visitor, call, queue_index, job_index, options);
    }
    if let Some(&(name_index, processor_index)) = visitor.worker_factories.get(local) {
        enqueue::note_worker(visitor, call, name_index, processor_index);
    }
}

pub(in crate::queue) fn scheduler_factories(
    program: &Program<'_>,
    roles: &QueueFactoryRoles,
) -> FxHashMap<SymbolId, usize> {
    let mut factories = fx_map();
    for statement in &program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        if import.import_kind.is_type() {
            continue;
        }
        for specifier in import.specifiers.iter().flatten() {
            let ImportDeclarationSpecifier::ImportSpecifier(specifier) = specifier else {
                continue;
            };
            if specifier.import_kind.is_type() {
                continue;
            }
            let imported = export_name(&specifier.imported);
            if let (Some(symbol), Some(role)) = (
                specifier.local.symbol_id.get(),
                roles
                    .job_scheduler
                    .iter()
                    .find(|role| role.callee == imported),
            ) {
                factories.insert(symbol, role.template);
            }
        }
    }
    factories
}

pub(crate) fn note_wait(visitor: &mut QueueVisitor<'_>, call: &CallExpression<'_>) {
    visitor.facts.checks.push(LocalCheck {
        line: line_number(visitor.source, call.span.start),
        kind: "enqueue-and-wait".to_string(),
        job: None,
        message: "waiting on a job result holds a worker or request slot across queues".to_string(),
    });
}
