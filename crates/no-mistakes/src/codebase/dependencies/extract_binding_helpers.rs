fn binding_names(pattern: &BindingPattern<'_>) -> Vec<String> {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => vec![identifier.name.to_string()],
        BindingPattern::ObjectPattern(object) => object
            .properties
            .iter()
            .flat_map(|property| binding_names(&property.value))
            .collect(),
        BindingPattern::ArrayPattern(array) => array
            .elements
            .iter()
            .flatten()
            .flat_map(binding_names)
            .collect(),
        BindingPattern::AssignmentPattern(assignment) => binding_names(&assignment.left),
    }
}

fn function_name(function: &oxc::ast::ast::Function<'_>) -> Option<String> {
    let id = function.id.as_ref()?;
    Some(id.name.to_string())
}

fn exported_top_level_binding(collector: &ImportCollector, name: Option<&String>) -> bool {
    name.is_some() && collector.function_stack.is_empty() && collector.export_depth > 0
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
    let pushed = name.is_some();
    collector.push_function_scope(name);
    let saved_suppress_imports = collector.suppress_imports;
    collector.suppress_imports = true;
    walk::walk_variable_declarator(collector, declarator);
    collector.suppress_imports = saved_suppress_imports;
    collector.pop_function_scope(pushed);
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
        collector.push_function_scope(Some(name));
        let saved_suppress_imports = collector.suppress_imports;
        collector.suppress_imports = true;
        if let Some(init) = &declarator.init {
            collector.visit_expression(init);
        }
        walk_variable_type_annotation(collector, declarator);
        collector.suppress_imports = saved_suppress_imports;
        collector.pop_function_scope(true);
    }
    true
}

impl ImportCollector {
    fn should_record_call(&self, callee: &str) -> bool {
        let binding = callee.split_once('.').map_or(callee, |(binding, _)| binding);
        if self.local_binding_shadows(binding) {
            self.has_local_function_scope(binding)
        } else {
            true
        }
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
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                    self.imported_bindings
                        .insert(specifier.local.name.to_string());
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                    self.imported_bindings
                        .insert(specifier.local.name.to_string());
                }
            }
        }
    }
}
