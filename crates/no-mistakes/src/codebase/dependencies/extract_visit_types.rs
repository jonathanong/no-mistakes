fn visit_exported_type_alias_declaration<'a>(
    collector: &mut ImportCollector,
    declaration: &TSTypeAliasDeclaration<'a>,
) {
    let pushed = true;
    let name = declaration.id.name.to_string();
    collector.exported_type_scopes.insert(name.clone());
    collector.push_function_scope(Some(name));
    collector.add_type_parameter_names(declaration.type_parameters.as_deref());
    walk::walk_ts_type_alias_declaration(collector, declaration);
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
    walk::walk_ts_interface_declaration(collector, declaration);
    collector.pop_function_scope(pushed);
}
