pub fn find_create_queue_line(
    source: &str,
    factory_specifier: &str,
    factory_function: &str,
) -> Option<u32> {
    let allocator = Allocator::default();
    let ret = crate::ast::parse(
        Path::new("queue-create.ts"),
        &allocator,
        source,
        SourceType::ts(),
    );
    find_create_queue_line_from_program(&ret.program, source, factory_specifier, factory_function)
}

pub fn find_create_queue_line_from_program(
    program: &Program<'_>,
    source: &str,
    factory_specifier: &str,
    factory_function: &str,
) -> Option<u32> {
    find_queue_factory_facts_from_program(program, source, factory_specifier, factory_function).0
}
