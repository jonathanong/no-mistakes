fn visit_binding_defaults_for_name<'a>(
    collector: &mut ImportCollector,
    pattern: &BindingPattern<'a>,
    name: &str,
) {
    match pattern {
        BindingPattern::AssignmentPattern(assignment) => {
            if binding_names(&assignment.left)
                .iter()
                .any(|binding| binding == name)
            {
                collector.visit_expression(&assignment.right);
            }
            visit_binding_defaults_for_name(collector, &assignment.left, name);
        }
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                visit_binding_defaults_for_name(collector, &property.value, name);
            }
        }
        BindingPattern::ArrayPattern(array) => {
            for element in array.elements.iter().flatten() {
                visit_binding_defaults_for_name(collector, element, name);
            }
        }
        BindingPattern::BindingIdentifier(_) => {}
    }
}

fn function_name(function: &oxc_ast::ast::Function<'_>) -> Option<String> {
    let id = function.id.as_ref()?;
    Some(id.name.to_string())
}

fn exported_top_level_binding(collector: &ImportCollector, name: Option<&String>) -> bool {
    name.is_some_and(|name| collector.is_exported_top_level_name(name))
        && collector.function_stack.is_empty()
}

fn walk_variable_type_annotation<'a>(
    collector: &mut ImportCollector,
    declarator: &VariableDeclarator<'a>,
) {
    if let Some(type_annotation) = &declarator.type_annotation {
        walk::walk_ts_type_annotation(collector, type_annotation);
    }
}

fn push_variable_function_scope<'a>(
    collector: &mut ImportCollector,
    declarator: &VariableDeclarator<'a>,
    name: Option<String>,
) {
    if let Some(name) = name.as_deref() {
        collector.record_callable_binding(name);
    }
    if exported_top_level_binding(collector, name.as_ref()) {
        collector.push_function_scope(name);
        if let Some(scope) = collector.current_function() {
            collector.exported_functions.insert(scope.clone());
            collector.callable_scopes.insert(scope);
        }
        walk::walk_binding_pattern(collector, &declarator.id);
        walk_variable_type_annotation(collector, declarator);
    } else if name.is_some() {
        walk::walk_binding_pattern(collector, &declarator.id);
        collector.push_function_scope(name);
        if let Some(scope) = collector.current_function() {
            collector.callable_scopes.insert(scope);
        }
    } else {
        walk::walk_binding_pattern(collector, &declarator.id);
        collector.push_anonymous_function_scope();
    }
}

fn visit_exported_variable_declarator_reference<'a>(
    collector: &mut ImportCollector,
    declarator: &VariableDeclarator<'a>,
    name: Option<String>,
) {
    let source_owner = (!matches!(
        declarator.init,
        Some(Expression::ObjectExpression(_) | Expression::ClassExpression(_))
    ))
    .then_some(name.clone())
    .flatten();
    let pushed_syntactic_caller = collector.push_syntactic_caller(source_owner);
    let pushed = name.is_some();
    collector.push_function_scope(name);
    let saved_suppress_imports = collector.suppress_imports;
    let saved_collect_runtime = collector.collect_suppressed_runtime_imports;
    let saved_base_depth = collector.runtime_reachable_base_depth;
    collector.suppress_imports = true;
    collector.collect_suppressed_runtime_imports = collector
        .current_function()
        .is_some_and(|scope| collector.is_exported_top_level_name(&scope));
    collector.runtime_reachable_base_depth = Some(collector.function_stack.len());
    walk::walk_variable_declarator(collector, declarator);
    collector.suppress_imports = saved_suppress_imports;
    collector.collect_suppressed_runtime_imports = saved_collect_runtime;
    collector.runtime_reachable_base_depth = saved_base_depth;
    collector.pop_function_scope(pushed);
    collector.pop_syntactic_caller(pushed_syntactic_caller);
}

fn visit_variable_declarator_references_for_bindings<'a>(
    collector: &mut ImportCollector,
    declarator: &VariableDeclarator<'a>,
) -> bool {
    let names = binding_names(&declarator.id);
    if names.is_empty() {
        return false;
    }
    for name in names {
        collector.push_function_scope(Some(name.clone()));
        let saved_suppress_imports = collector.suppress_imports;
        collector.suppress_imports = true;
        if let Some(init) = &declarator.init {
            collector.visit_expression(init);
        }
        visit_binding_defaults_for_name(collector, &declarator.id, &name);
        walk_variable_type_annotation(collector, declarator);
        collector.suppress_imports = saved_suppress_imports;
        collector.pop_function_scope(true);
    }
    true
}

impl ImportCollector {
    fn should_record_call(&self, _callee: &str) -> bool {
        // A shadowed value still represents a real callsite. Its target is
        // Unknown unless we can resolve a local callable scope; dropping it
        // would make policy checks silently miss dynamic/local dispatch.
        true
    }

    fn record_imported_bindings(&mut self, import: &ImportDeclaration<'_>) {
        let Some(specifiers) = &import.specifiers else {
            return;
        };
        for specifier in specifiers {
            match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                    self.imported_bindings
                        .insert(specifier.local.name.to_string());
                    self.call_import_bindings.push(ImportedBinding {
                        specifier: import.source.value.to_string(),
                        local: specifier.local.name.to_string(),
                        imported: specifier.imported.name().to_string(),
                        kind: ImportedBindingKind::Named,
                        is_type_only: import.import_kind.is_type()
                            || specifier.import_kind.is_type(),
                    });
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                    self.imported_bindings
                        .insert(specifier.local.name.to_string());
                    self.call_import_bindings.push(ImportedBinding {
                        specifier: import.source.value.to_string(),
                        local: specifier.local.name.to_string(),
                        imported: "default".to_string(),
                        kind: ImportedBindingKind::Default,
                        is_type_only: import.import_kind.is_type(),
                    });
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                    self.imported_bindings
                        .insert(specifier.local.name.to_string());
                    self.call_import_bindings.push(ImportedBinding {
                        specifier: import.source.value.to_string(),
                        local: specifier.local.name.to_string(),
                        imported: "*".to_string(),
                        kind: ImportedBindingKind::Namespace,
                        is_type_only: import.import_kind.is_type(),
                    });
                }
            }
        }
    }
}
