fn predeclare_program_value_bindings<'a>(collector: &mut ImportCollector, program: &Program<'a>) {
    for statement in &program.body {
        predeclare_hoisted_function_identity(collector, statement);
    }
    for statement in &program.body {
        predeclare_program_statement_binding(collector, statement);
    }
    predeclare_hoisted_var_bindings(collector, &program.body);
}

fn predeclare_hoisted_function_identity(
    collector: &mut ImportCollector,
    statement: &Statement<'_>,
) {
    let function = match statement {
        Statement::FunctionDeclaration(function) => Some(function.as_ref()),
        Statement::ExportDeclaration(export) => match &export.declaration {
            Declaration::FunctionDeclaration(function) => Some(function.as_ref()),
            _ => None,
        },
        _ => None,
    };
    let Some(function) = function.filter(|function| function.body.is_some()) else {
        return;
    };
    if let Some(name) = function_name(function) {
        collector.record_callable_binding_id(&name, CallableId(function.span.start));
    }
}

fn predeclare_program_statement_binding(
    collector: &mut ImportCollector,
    statement: &Statement<'_>,
) {
    match statement {
        Statement::FunctionDeclaration(function) => {
            predeclare_function_binding(collector, function);
        }
        Statement::VariableDeclaration(declaration) => {
            predeclare_variable_bindings(collector, declaration);
        }
        Statement::ClassDeclaration(class) => predeclare_class_binding(collector, class),
        Statement::ImportDeclaration(import) => predeclare_import_bindings(collector, import),
        Statement::ExportDeclaration(export) => {
            predeclare_export_declaration_binding(collector, &export.declaration);
        }
        Statement::ExportDefaultDeclaration(export) => {
            predeclare_default_export_binding(collector, &export.declaration);
        }
        _ => {}
    }
}

fn predeclare_function_binding(collector: &mut ImportCollector, function: &Function<'_>) {
    let Some(name) = function_name(function) else {
        return;
    };
    collector.add_binding_name(&name);
    if collector.callable_binding_id(&name).is_none() {
        collector.record_callable_binding_id(&name, CallableId(function.span.start));
    }
    collector.known_function_scopes.insert(name.clone());
    collector.callable_scopes.insert(name);
}

fn predeclare_variable_bindings(
    collector: &mut ImportCollector,
    declaration: &VariableDeclaration<'_>,
) {
    for declarator in &declaration.declarations {
        collector.add_binding_names(&declarator.id);
    }
}

fn predeclare_class_binding(collector: &mut ImportCollector, class: &Class<'_>) {
    if let Some(name) = class.id.as_ref() {
        collector.add_binding_name(name.name.as_str());
    }
}

fn predeclare_import_bindings(collector: &mut ImportCollector, import: &ImportDeclaration<'_>) {
    if import.import_kind.is_type() {
        return;
    }
    let Some(specifiers) = &import.specifiers else {
        return;
    };
    for specifier in specifiers {
        let local = match specifier {
            ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                if specifier.import_kind.is_type() {
                    continue;
                }
                &specifier.local
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => &specifier.local,
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => &specifier.local,
        };
        collector
            .predeclared_imported_bindings
            .insert(local.name.to_string());
    }
}

fn predeclare_default_export_binding(
    collector: &mut ImportCollector,
    declaration: &ExportDefaultDeclarationKind<'_>,
) {
    match declaration {
        ExportDefaultDeclarationKind::FunctionDeclaration(function) => {
            predeclare_function_binding(collector, function);
        }
        ExportDefaultDeclarationKind::ClassDeclaration(class) => {
            predeclare_class_binding(collector, class);
        }
        _ => {}
    }
}

fn predeclare_export_declaration_binding<'a>(
    collector: &mut ImportCollector,
    declaration: &Declaration<'a>,
) {
    match declaration {
        Declaration::FunctionDeclaration(function) => {
            predeclare_function_binding(collector, function);
        }
        Declaration::VariableDeclaration(declaration) => {
            predeclare_variable_bindings(collector, declaration);
        }
        Declaration::ClassDeclaration(class) => {
            predeclare_class_binding(collector, class);
        }
        _ => {}
    }
}
