use super::*;

pub(super) fn visit<'a>(collector: &mut Collector<'_>, module: &TSExternalModuleDeclaration<'a>) {
    let prior = collector
        .external_module
        .replace(module.id.value.to_string());
    if collector.global_modules
        && prior.is_none()
        && collector.function_depth == 0
        && collector.namespace.is_empty()
    {
        if let Some(body) = &module.body {
            let exported = super::super::ancestry_namespaces::collect(body, &[], true)
                .values
                .into_iter()
                .filter(|value| value.exported)
                .filter_map(|value| value.class)
                .collect();
            collector
                .facts
                .ambient_modules
                .push((module.id.value.to_string(), exported));
        }
    }
    walk::walk_ts_external_module_declaration(collector, module);
    collector.external_module = prior;
}

pub(super) fn is_script(program: &Program<'_>) -> bool {
    !program.body.iter().any(|statement| match statement {
        // Only external import-equals creates a module boundary. An internal
        // namespace alias leaves script-level ambient modules global.
        Statement::TSImportEqualsDeclaration(import) => matches!(
            &import.module_reference,
            TSModuleReference::ExternalModuleReference(_)
        ),
        statement => matches!(
            statement,
            Statement::ImportDeclaration(_)
                | Statement::ExportDeclaration(_)
                | Statement::ExportNamedDeclaration(_)
                | Statement::ExportAllDeclaration(_)
                | Statement::ExportFromDeclaration(_)
                | Statement::ExportDefaultDeclaration(_)
                | Statement::TSExportAssignment(_)
        ),
    })
}
