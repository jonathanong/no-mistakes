use crate::fx::FxHashMap;
use crate::queue::extract_helpers::binding_name;
use crate::queue::extract_visitor::QueueVisitor;
use crate::queue::source::line_number;
use oxc_ast::ast::{
    BindingIdentifier, BindingPattern, CallExpression, Declaration, ExportDefaultDeclaration,
    ExportDefaultDeclarationKind, Program, Statement, VariableDeclaration, VariableDeclarationKind,
    VariableDeclarator,
};
use oxc_syntax::symbol::SymbolId;
use std::collections::HashMap;

mod classify;
mod finish;
mod predicates;

pub(super) use predicates::is_worker_constructor_callee;

#[derive(Default)]
pub(super) struct State {
    pub(super) worker_call_indices: HashMap<u32, usize>,
    pub(super) local_bindings: FxHashMap<SymbolId, LocalInit>,
    pub(super) module_bindings: HashMap<String, ModuleInit>,
    pub(super) pending_calls: Vec<PendingCall>,
}

#[derive(Clone)]
pub(super) enum LocalInit {
    WorkerSpan(u32),
    NonWorker,
    Alias(SymbolId),
    Imported(String),
    Namespace { source: String, exported: String },
}

#[derive(Clone)]
pub(super) enum ModuleInit {
    WorkerSpan(u32),
    NonWorker,
    Alias(String),
    Imported(String),
}

pub(super) struct PendingCall {
    pub(super) line: usize,
    pub(super) receiver: PendingReceiver,
}

pub(super) enum PendingReceiver {
    WorkerSpan(u32),
    Local(SymbolId),
    Imported(String),
    Namespace { source: String, exported: String },
    Unresolved,
}

impl State {
    pub(super) fn note_worker_span(&mut self, span_start: u32, worker_index: usize) {
        self.worker_call_indices.insert(span_start, worker_index);
    }
}

pub(super) fn note_variable(visitor: &mut QueueVisitor<'_>, decl: &VariableDeclarator<'_>) {
    let Some(init) = &decl.init else {
        return;
    };
    if let (Some(scoping), BindingPattern::BindingIdentifier(id)) = (visitor.scoping, &decl.id) {
        if let Some(symbol) = id.symbol_id.get() {
            if !scoping.symbol_is_mutated(symbol)
                && scoping.symbol_redeclarations(symbol).is_empty()
            {
                if let Some(local_init) = classify::classify_local_init(visitor, init) {
                    visitor
                        .worker_rate_limit
                        .local_bindings
                        .insert(symbol, local_init);
                }
            }
        }
    }
}

pub(super) fn note_export_default(
    visitor: &mut QueueVisitor<'_>,
    export: &ExportDefaultDeclaration<'_>,
) {
    let init = match &export.declaration {
        ExportDefaultDeclarationKind::FunctionDeclaration(_)
        | ExportDefaultDeclarationKind::ClassDeclaration(_) => Some(ModuleInit::NonWorker),
        kind => kind
            .as_expression()
            .and_then(|expr| classify::classify_module_init(visitor, expr)),
    };
    if let Some(module_init) = init {
        visitor
            .worker_rate_limit
            .module_bindings
            .insert("default".to_string(), module_init);
    }
}

pub(super) fn note_call(visitor: &mut QueueVisitor<'_>, call: &CallExpression<'_>) {
    let Some(object) = classify::rate_limit_receiver(call) else {
        return;
    };
    let Some(receiver) = classify::classify_receiver(visitor, object) else {
        return;
    };
    visitor.worker_rate_limit.pending_calls.push(PendingCall {
        line: line_number(visitor.source, call.span.start),
        receiver,
    });
}

pub(super) fn finish(visitor: &mut QueueVisitor<'_>, program: &Program<'_>) {
    record_top_level_decls(visitor, program);
    finish::finish(visitor);
}

fn record_top_level_decls(visitor: &mut QueueVisitor<'_>, program: &Program<'_>) {
    for stmt in &program.body {
        match stmt {
            Statement::VariableDeclaration(var_decl) => record_module_consts(visitor, var_decl),
            Statement::FunctionDeclaration(func) => note_non_worker_id(visitor, func.id.as_ref()),
            Statement::ClassDeclaration(class) => note_non_worker_id(visitor, class.id.as_ref()),
            Statement::ExportDeclaration(export) => match &export.declaration {
                Declaration::VariableDeclaration(var_decl) => {
                    record_module_consts(visitor, var_decl)
                }
                Declaration::FunctionDeclaration(func) => {
                    note_non_worker_id(visitor, func.id.as_ref())
                }
                Declaration::ClassDeclaration(class) => {
                    note_non_worker_id(visitor, class.id.as_ref())
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn note_non_worker_id(visitor: &mut QueueVisitor<'_>, id: Option<&BindingIdentifier<'_>>) {
    if let Some(id) = id {
        visitor
            .worker_rate_limit
            .module_bindings
            .insert(id.name.to_string(), ModuleInit::NonWorker);
    }
}

fn record_module_consts(visitor: &mut QueueVisitor<'_>, decl: &VariableDeclaration<'_>) {
    if decl.kind != VariableDeclarationKind::Const {
        return;
    }
    for declarator in &decl.declarations {
        let (Some(name), Some(init)) = (binding_name(&declarator.id), declarator.init.as_ref())
        else {
            continue;
        };
        if let Some(module_init) = classify::classify_module_init(visitor, init) {
            visitor
                .worker_rate_limit
                .module_bindings
                .insert(name, module_init);
        }
    }
}
