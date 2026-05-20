#[derive(Debug, Clone)]
pub struct RouteRef {
    pub pattern: String,
    pub file: String,
    pub line: u32,
}

/// Scan `source` for route references. Returns a Vec of RouteRef.
pub fn extract_route_refs(source: &str, file: &str) -> Vec<RouteRef> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(Path::new(file)).unwrap_or(SourceType::tsx());
    let ret = Parser::new(&allocator, source, source_type).parse();

    extract_route_refs_from_program(&ret.program, source, file)
}

pub fn extract_route_refs_from_program<'a>(
    program: &Program<'a>,
    source: &str,
    file: &str,
) -> Vec<RouteRef> {
    let mut router_bindings = collect_import_bindings(&program.body);
    collect_router_bindings_for_scope(&program.body, &mut router_bindings);

    let mut refs = Vec::new();
    for stmt in &program.body {
        collect_from_statement(stmt, source, file, &mut router_bindings, &mut refs);
    }

    refs
}

#[derive(Clone, Default)]
struct RouterBindings<'a> {
    objects: HashSet<&'a str>,
    methods: HashSet<&'a str>,
    redirects: HashSet<&'a str>,
}
