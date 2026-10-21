use super::{
    binding_identifier_name, binding_names, function_name, record_statement_type_binding,
    visit_type_parameter_constraints, with_callable_receiver_execution, CallableId,
    ImportCollector,
};
use crate::fx::{fx_set, FxHashSet};
use oxc_ast::ast::{
    Class, Expression, Statement, TSModuleBlock, VariableDeclaration, VariableDeclarationKind,
};
use oxc_ast_visit::{walk, Visit};

pub(super) fn walk_function_with_body_bindings<'a>(
    collector: &mut ImportCollector,
    function: &oxc_ast::ast::Function<'a>,
    owner_entered: bool,
) {
    let enclosing_unreachable_binding_after = collector
        .collect_value_flow
        .then(|| collector.unreachable_value_binding_after.take());
    with_callable_receiver_execution(collector, |collector| {
        let previous_repeat_base = collector.call_repeat_base;
        if owner_entered {
            collector.call_repeat_base = collector.repeat_depth;
        }
        collector.record_callable_result_kind(function.r#async, function.generator);
        // Default parameters run in the parameter environment, before the body
        // lexical environment exists. Visit them before introducing body bindings.
        visit_type_parameter_constraints(collector, function.type_parameters.as_deref());
        walk::walk_formal_parameters(collector, &function.params);
        if let Some(return_type) = &function.return_type {
            walk::walk_ts_type_annotation(collector, return_type);
        }
        if let Some(body) = &function.body {
            let previous_cutoff = collector.record_value_fallthrough(&body.statements);
            predeclare_function_declarations(collector, &body.statements);
            walk::walk_function_body(collector, body);
            collector.unreachable_value_binding_after = previous_cutoff;
        }
        collector.call_repeat_base = previous_repeat_base;
    });
    if let Some(previous) = enclosing_unreachable_binding_after {
        collector.unreachable_value_binding_after = previous;
    }
}

pub(super) fn walk_arrow_function_with_body_bindings<'a>(
    collector: &mut ImportCollector,
    arrow: &oxc_ast::ast::ArrowFunctionExpression<'a>,
    owner_entered: bool,
) {
    collector.note_arrow_scope();
    let enclosing_unreachable_binding_after = collector
        .collect_value_flow
        .then(|| collector.unreachable_value_binding_after.take());
    with_callable_receiver_execution(collector, |collector| {
        let previous_repeat_base = collector.call_repeat_base;
        if owner_entered {
            collector.call_repeat_base = collector.repeat_depth;
        }
        collector.record_callable_result_kind(arrow.r#async, false);
        // Default parameters run before the body lexical environment exists.
        visit_type_parameter_constraints(collector, arrow.type_parameters.as_deref());
        walk::walk_formal_parameters(collector, &arrow.params);
        if let Some(return_type) = &arrow.return_type {
            walk::walk_ts_type_annotation(collector, return_type);
        }
        if let Some(body) = crate::ast::arrow_function_body(&arrow.body) {
            let previous_cutoff = collector.record_value_fallthrough(&body.statements);
            predeclare_function_declarations(collector, &body.statements);
            walk::walk_function_body(collector, body);
            collector.unreachable_value_binding_after = previous_cutoff;
        } else if let Some(expression) = arrow.body.as_expression() {
            if collector.collect_value_flow {
                if let Some(owner) = collector.current_function_id() {
                    let value = collector.value_expression(expression);
                    collector.value_flow.returns.push((owner, value));
                }
            }
            collector.visit_expression(expression);
        }
        collector.call_repeat_base = previous_repeat_base;
    });
    if let Some(previous) = enclosing_unreachable_binding_after {
        collector.unreachable_value_binding_after = previous;
    }
}

pub(super) fn predeclare_hoisted_var_bindings<'a>(
    collector: &mut ImportCollector,
    statements: &[Statement<'a>],
) {
    let mut names = fx_set();
    let mut visitor = HoistedVarBindingCollector { names: &mut names };
    for statement in statements {
        // Type declarations shadow imports throughout this lexical body.
        record_statement_type_binding(collector, statement);
        visitor.visit_statement(statement);
    }
    for name in names {
        collector.add_var_binding_name(&name);
    }
}

struct HoistedVarBindingCollector<'a> {
    names: &'a mut FxHashSet<String>,
}

impl<'ast> Visit<'ast> for HoistedVarBindingCollector<'_> {
    fn visit_variable_declaration(&mut self, declaration: &VariableDeclaration<'ast>) {
        if declaration.kind == VariableDeclarationKind::Var {
            for declarator in &declaration.declarations {
                self.names.extend(binding_names(&declarator.id));
            }
        }
        walk::walk_variable_declaration(self, declaration);
    }

    // Nested callables own their own `var` environments.
    fn visit_function(
        &mut self,
        _function: &oxc_ast::ast::Function<'ast>,
        _flags: oxc_syntax::scope::ScopeFlags,
    ) {
    }

    fn visit_arrow_function_expression(
        &mut self,
        _arrow: &oxc_ast::ast::ArrowFunctionExpression<'ast>,
    ) {
    }

    // Class static blocks own their own `var` environment, just like class
    // methods own their function environments. Neither can hoist into the
    // surrounding module or function.
    fn visit_class(&mut self, _class: &Class<'ast>) {}

    // A namespace body is a function-like scope: its `var`s stay inside it.
    fn visit_ts_module_block(&mut self, _block: &TSModuleBlock<'ast>) {}
}

pub(super) fn predeclare_function_declarations<'a>(
    collector: &mut ImportCollector,
    statements: &[Statement<'a>],
) {
    for statement in statements {
        record_statement_type_binding(collector, statement);
        let Statement::FunctionDeclaration(function) = statement else {
            continue;
        };
        if function.body.is_some() {
            if let Some(name) = function_name(function) {
                collector.record_callable_binding_id(&name, CallableId(function.span.start));
            }
        }
    }
    for statement in statements {
        match statement {
            Statement::FunctionDeclaration(function) => {
                if let Some(name) = function_name(function) {
                    collector.add_binding_name(&name);
                    if collector.callable_binding_id(&name).is_none() {
                        collector
                            .record_callable_binding_id(&name, CallableId(function.span.start));
                    }
                    let scope = collector.callable_scope_name(&name);
                    collector.known_function_scopes.insert(scope.clone());
                    collector.callable_scopes.insert(scope);
                }
            }
            // We only use this catalog to prove that a spelling is locally
            // bound. A call before a `let`/`const` declaration is a TDZ error
            // at runtime, but it must never be misclassified as a global API.
            Statement::VariableDeclaration(declaration) => {
                for declarator in &declaration.declarations {
                    if declaration.kind == VariableDeclarationKind::Var {
                        collector.add_var_binding_names(&declarator.id);
                    } else {
                        collector.add_binding_names(&declarator.id);
                    }
                    collector.note_predeclared_array_binding(&declarator.id, declaration.kind);
                }
            }
            Statement::ClassDeclaration(class) => {
                if let Some(name) = class.id.as_ref() {
                    collector.add_binding_name(name.name.as_str());
                }
            }
            _ => {}
        }
    }
    predeclare_hoisted_var_bindings(collector, statements);
}

#[path = "extract_visit_callable_declarations.rs"]
mod extract_visit_callable_declarations;
