/// Parse `source` and return the queue name from a `createQueue(name, ...)` call.
/// Resolves top-level `const NAME = "..."` bindings when the first argument is an identifier.
/// Returns `Some("<unknown>")` when the call is found but the name cannot be statically resolved.
pub fn find_queue_name(
    source: &str,
    factory_specifier: &str,
    factory_function: &str,
) -> Option<String> {
    let allocator = Allocator::default();
    let ret = crate::ast::parse(
        Path::new("queue-name.ts"),
        &allocator,
        source,
        SourceType::ts(),
    );
    find_queue_name_from_program(&ret.program, factory_specifier, factory_function)
}

pub fn find_queue_name_from_program(
    program: &Program<'_>,
    factory_specifier: &str,
    factory_function: &str,
) -> Option<String> {
    find_queue_factory_facts_from_program(program, "", factory_specifier, factory_function).1
}

fn resolve_queue_name(
    call: &oxc_ast::ast::CallExpression<'_>,
    const_strings: &HashMap<String, String>,
) -> String {
    match call.arguments.first() {
        Some(oxc_ast::ast::Argument::StringLiteral(s)) => crate::ast::js_literal(&s).into_owned(),
        Some(oxc_ast::ast::Argument::Identifier(id)) => const_strings
            .get(id.name.as_str())
            .cloned()
            .unwrap_or_else(|| "<unknown>".to_string()),
        _ => "<unknown>".to_string(),
    }
}

fn module_export_name_str(name: &ModuleExportName) -> String {
    crate::ast::module_export_name(name).into_owned()
}
