fn predeclare_program_value_bindings<'a>(collector: &mut ImportCollector, program: &Program<'a>) {
    for statement in &program.body {
        match statement {
            Statement::FunctionDeclaration(function) => {
                if let Some(name) = function_name(function) {
                    collector.add_binding_name(&name);
                    collector.record_callable_binding(&name);
                    collector.known_function_scopes.insert(name.clone());
                    collector.callable_scopes.insert(name);
                }
            }
            Statement::VariableDeclaration(declaration) => {
                for declarator in &declaration.declarations {
                    collector.add_binding_names(&declarator.id);
                }
            }
            Statement::ClassDeclaration(class) => {
                if let Some(name) = class.id.as_ref() {
                    collector.add_binding_name(name.name.as_str());
                }
            }
            Statement::ImportDeclaration(import) => {
                if import.import_kind.is_type() {
                    continue;
                }
                if let Some(specifiers) = &import.specifiers {
                    for specifier in specifiers {
                        match specifier {
                            ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                                if specifier.import_kind.is_type() {
                                    continue;
                                }
                                collector
                                    .predeclared_imported_bindings
                                    .insert(specifier.local.name.to_string());
                            }
                            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                                collector
                                    .predeclared_imported_bindings
                                    .insert(specifier.local.name.to_string());
                            }
                            ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                                collector
                                    .predeclared_imported_bindings
                                    .insert(specifier.local.name.to_string());
                            }
                        }
                    }
                }
            }
            Statement::ExportDeclaration(export) => {
                predeclare_export_declaration_binding(collector, &export.declaration);
            }
            Statement::ExportDefaultDeclaration(export) => match &export.declaration {
                ExportDefaultDeclarationKind::FunctionDeclaration(function) => {
                    if let Some(name) = function_name(function) {
                        collector.add_binding_name(&name);
                    }
                }
                ExportDefaultDeclarationKind::ClassDeclaration(class) => {
                    if let Some(name) = class.id.as_ref() {
                        collector.add_binding_name(name.name.as_str());
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
    predeclare_hoisted_var_bindings(collector, &program.body);
}

fn predeclare_export_declaration_binding<'a>(
    collector: &mut ImportCollector,
    declaration: &Declaration<'a>,
) {
    match declaration {
        Declaration::FunctionDeclaration(function) => {
            if let Some(name) = function_name(function) {
                collector.add_binding_name(&name);
                collector.known_function_scopes.insert(name.clone());
                collector.callable_scopes.insert(name);
            }
        }
        Declaration::VariableDeclaration(declaration) => {
            for declarator in &declaration.declarations {
                collector.add_binding_names(&declarator.id);
            }
        }
        Declaration::ClassDeclaration(class) => {
            if let Some(name) = class.id.as_ref() {
                collector.add_binding_name(name.name.as_str());
            }
        }
        _ => {}
    }
}
