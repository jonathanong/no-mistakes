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
