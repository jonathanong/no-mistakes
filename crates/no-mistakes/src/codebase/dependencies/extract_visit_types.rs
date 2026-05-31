fn visit_exported_type_alias_declaration<'a>(
    collector: &mut ImportCollector,
    declaration: &TSTypeAliasDeclaration<'a>,
) {
    let pushed = true;
    let name = declaration.id.name.to_string();
    collector.exported_type_scopes.insert(name.clone());
    collector.push_function_scope(Some(name));
    collector.add_type_parameter_names(declaration.type_parameters.as_deref());
    collector.visit_ts_type(&declaration.type_annotation);
    collector.pop_function_scope(pushed);
}

fn visit_exported_interface_declaration<'a>(
    collector: &mut ImportCollector,
    declaration: &TSInterfaceDeclaration<'a>,
) {
    let pushed = true;
    let name = declaration.id.name.to_string();
    collector.exported_type_scopes.insert(name.clone());
    collector.push_function_scope(Some(name));
    collector.add_type_parameter_names(declaration.type_parameters.as_deref());
    collector.visit_ts_interface_heritages(&declaration.extends);
    collector.visit_ts_interface_body(&declaration.body);
    collector.pop_function_scope(pushed);
}
