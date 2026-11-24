use crate::queue::extract_helpers::binding_name;
use crate::queue::extract_visitor::QueueVisitor;
use crate::queue::source::line_number;
use oxc_ast::ast::{
    AssignmentExpression, AssignmentTarget, BindingIdentifier, BindingPattern, CallExpression,
    Class, Declaration, ExportDefaultDeclaration, ExportDefaultDeclarationKind, Program, Statement,
    VariableDeclaration, VariableDeclarationKind, VariableDeclarator,
};

mod classes;
mod classify;
mod classify_module;
mod finish;
mod finish_exports;
mod predicates;
mod state;

pub(super) use predicates::is_worker_constructor_callee;
pub(super) use state::State;
use state::{LocalClassInit, LocalInit, ModuleClassInit, ModuleInit, PendingCall, PendingReceiver};

pub(super) fn note_class(visitor: &mut QueueVisitor<'_>, class: &Class<'_>) {
    let Some(id) = &class.id else {
        return;
    };
    let Some(symbol) = id.symbol_id.get() else {
        return;
    };
    let init = classes::classify_local_class(visitor, class);
    visitor.worker_rate_limit.local_classes.insert(symbol, init);
}

pub(super) fn note_variable(visitor: &mut QueueVisitor<'_>, decl: &VariableDeclarator<'_>) {
    let Some(init) = &decl.init else {
        return;
    };
    let (Some(scoping), BindingPattern::BindingIdentifier(id)) = (visitor.scoping, &decl.id) else {
        return;
    };
    let Some(symbol) = id.symbol_id.get() else {
        return;
    };
    let Some(local_init) = classify::classify_local_init(visitor, init) else {
        return;
    };
    if !scoping.symbol_is_mutated(symbol) && scoping.symbol_redeclarations(symbol).is_empty() {
        visitor
            .worker_rate_limit
            .local_bindings
            .insert(symbol, local_init);
    } else if !matches!(local_init, LocalInit::NonWorker) {
        visitor
            .worker_rate_limit
            .mutated_origins
            .entry(symbol)
            .or_default()
            .push(local_init);
    }
}

pub(super) fn note_assignment(visitor: &mut QueueVisitor<'_>, expr: &AssignmentExpression<'_>) {
    let AssignmentTarget::AssignmentTargetIdentifier(id) = &expr.left else {
        return;
    };
    let Some(symbol) = predicates::referenced_symbol(visitor, id) else {
        return;
    };
    let Some(local_init) = classify::classify_local_init(visitor, &expr.right) else {
        return;
    };
    if !matches!(local_init, LocalInit::NonWorker) {
        visitor
            .worker_rate_limit
            .mutated_origins
            .entry(symbol)
            .or_default()
            .push(local_init);
    }
}

pub(super) fn note_export_default(
    visitor: &mut QueueVisitor<'_>,
    export: &ExportDefaultDeclaration<'_>,
) {
    match &export.declaration {
        ExportDefaultDeclarationKind::FunctionDeclaration(_) => {
            visitor
                .worker_rate_limit
                .module_bindings
                .insert("default".to_string(), ModuleInit::NonWorker);
        }
        ExportDefaultDeclarationKind::ClassDeclaration(class) => {
            let class_init = classes::classify_module_class(visitor, class);
            visitor
                .worker_rate_limit
                .module_classes
                .insert("default".to_string(), class_init);
        }
        kind => {
            if let Some(module_init) = kind
                .as_expression()
                .and_then(|expr| classify_module::classify_module_init(visitor, expr))
            {
                visitor
                    .worker_rate_limit
                    .module_bindings
                    .insert("default".to_string(), module_init);
            }
        }
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
            Statement::ClassDeclaration(class) => record_module_class(visitor, class),
            Statement::ExportDeclaration(export) => match &export.declaration {
                Declaration::VariableDeclaration(var_decl) => {
                    record_module_consts(visitor, var_decl)
                }
                Declaration::FunctionDeclaration(func) => {
                    note_non_worker_id(visitor, func.id.as_ref())
                }
                Declaration::ClassDeclaration(class) => record_module_class(visitor, class),
                _ => {}
            },
            _ => {}
        }
    }
}

fn record_module_class(visitor: &mut QueueVisitor<'_>, class: &Class<'_>) {
    let Some(id) = &class.id else {
        return;
    };
    let class_init = classes::classify_module_class(visitor, class);
    visitor
        .worker_rate_limit
        .module_classes
        .insert(id.name.to_string(), class_init);
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
        if let Some(module_init) = classify_module::classify_module_init(visitor, init) {
            visitor
                .worker_rate_limit
                .module_bindings
                .insert(name, module_init);
        }
    }
}
