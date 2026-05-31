fn visit_ts_type_alias_declaration_with_scope<'a>(
    collector: &mut ImportCollector,
    declaration: &TSTypeAliasDeclaration<'a>,
) {
    if collector.function_stack.is_empty()
        && collector.is_exported_top_level_type_name(declaration.id.name.as_str())
    {
        visit_exported_type_alias_declaration(collector, declaration);
    } else if collector.function_stack.is_empty() {
        collector.add_type_binding_name(declaration.id.name.as_str());
        visit_type_alias_declaration_with_scope(collector, declaration, true);
        visit_type_alias_declaration_file_fallback(collector, declaration);
    } else {
        collector.add_type_binding_name(declaration.id.name.as_str());
        collector.add_type_parameter_names(declaration.type_parameters.as_deref());
        collector.visit_ts_type(&declaration.type_annotation);
    }
}

fn visit_ts_interface_declaration_with_scope<'a>(
    collector: &mut ImportCollector,
    declaration: &TSInterfaceDeclaration<'a>,
) {
    if collector.function_stack.is_empty()
        && collector.is_exported_top_level_type_name(declaration.id.name.as_str())
    {
        visit_exported_interface_declaration(collector, declaration);
    } else if collector.function_stack.is_empty() {
        collector.add_type_binding_name(declaration.id.name.as_str());
        visit_interface_declaration_with_scope(collector, declaration, true);
        visit_interface_declaration_file_fallback(collector, declaration);
    } else {
        collector.add_type_binding_name(declaration.id.name.as_str());
        collector.add_type_parameter_names(declaration.type_parameters.as_deref());
        collector.visit_ts_interface_heritages(&declaration.extends);
        collector.visit_ts_interface_body(&declaration.body);
    }
}

fn visit_exported_type_alias_declaration<'a>(
    collector: &mut ImportCollector,
    declaration: &TSTypeAliasDeclaration<'a>,
) {
    let name = declaration.id.name.to_string();
    collector.exported_type_scopes.insert(name.clone());
    visit_type_alias_declaration_with_scope_name(collector, declaration, name, false);
}

fn visit_exported_interface_declaration<'a>(
    collector: &mut ImportCollector,
    declaration: &TSInterfaceDeclaration<'a>,
) {
    let name = declaration.id.name.to_string();
    collector.exported_type_scopes.insert(name.clone());
    visit_interface_declaration_with_scope_name(collector, declaration, name, false);
}

fn visit_type_alias_declaration_with_scope<'a>(
    collector: &mut ImportCollector,
    declaration: &TSTypeAliasDeclaration<'a>,
    suppress_imports: bool,
) {
    let name = declaration.id.name.to_string();
    visit_type_alias_declaration_with_scope_name(collector, declaration, name, suppress_imports);
}

fn visit_type_alias_declaration_with_scope_name<'a>(
    collector: &mut ImportCollector,
    declaration: &TSTypeAliasDeclaration<'a>,
    name: String,
    suppress_imports: bool,
) {
    collector.push_function_scope(Some(name));
    collector.add_type_parameter_names(declaration.type_parameters.as_deref());
    visit_type_parameter_constraints(collector, declaration.type_parameters.as_deref());
    let saved_suppress_imports = collector.suppress_imports;
    collector.suppress_imports = suppress_imports;
    collector.visit_ts_type(&declaration.type_annotation);
    collector.suppress_imports = saved_suppress_imports;
    collector.pop_function_scope(true);
}

fn visit_type_alias_declaration_file_fallback<'a>(
    collector: &mut ImportCollector,
    declaration: &TSTypeAliasDeclaration<'a>,
) {
    push_top_level_type_scope(collector);
    collector.add_type_binding_name(declaration.id.name.as_str());
    collector.add_type_parameter_names(declaration.type_parameters.as_deref());
    visit_type_parameter_constraints(collector, declaration.type_parameters.as_deref());
    collector.visit_ts_type(&declaration.type_annotation);
    pop_top_level_type_scope(collector);
}

fn visit_interface_declaration_with_scope<'a>(
    collector: &mut ImportCollector,
    declaration: &TSInterfaceDeclaration<'a>,
    suppress_imports: bool,
) {
    let name = declaration.id.name.to_string();
    visit_interface_declaration_with_scope_name(collector, declaration, name, suppress_imports);
}

fn visit_interface_declaration_with_scope_name<'a>(
    collector: &mut ImportCollector,
    declaration: &TSInterfaceDeclaration<'a>,
    name: String,
    suppress_imports: bool,
) {
    collector.push_function_scope(Some(name));
    collector.add_type_parameter_names(declaration.type_parameters.as_deref());
    visit_type_parameter_constraints(collector, declaration.type_parameters.as_deref());
    let saved_suppress_imports = collector.suppress_imports;
    collector.suppress_imports = suppress_imports;
    collector.visit_ts_interface_heritages(&declaration.extends);
    collector.visit_ts_interface_body(&declaration.body);
    collector.suppress_imports = saved_suppress_imports;
    collector.pop_function_scope(true);
}

fn visit_interface_declaration_file_fallback<'a>(
    collector: &mut ImportCollector,
    declaration: &TSInterfaceDeclaration<'a>,
) {
    push_top_level_type_scope(collector);
    collector.add_type_binding_name(declaration.id.name.as_str());
    collector.add_type_parameter_names(declaration.type_parameters.as_deref());
    visit_type_parameter_constraints(collector, declaration.type_parameters.as_deref());
    collector.visit_ts_interface_heritages(&declaration.extends);
    collector.visit_ts_interface_body(&declaration.body);
    pop_top_level_type_scope(collector);
}

fn visit_type_parameter_constraints<'a>(
    collector: &mut ImportCollector,
    params: Option<&TSTypeParameterDeclaration<'a>>,
) {
    let Some(params) = params else { return };
    for parameter in &params.params {
        collector.visit_ts_type_parameter(parameter);
    }
}

fn push_top_level_type_scope(collector: &mut ImportCollector) {
    collector.type_local_stack.push(HashSet::new());
    collector.type_parameter_stack.push(HashSet::new());
}

fn pop_top_level_type_scope(collector: &mut ImportCollector) {
    collector.type_local_stack.pop();
    collector.type_parameter_stack.pop();
}
