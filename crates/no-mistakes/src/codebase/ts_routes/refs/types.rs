#[derive(Debug, Clone, PartialEq)]
pub struct RouteRef {
    pub pattern: String,
    pub file: String,
    pub line: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteHelper {
    pub name: String,
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteHelperImport {
    pub local: String,
    pub imported: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteHelperRef {
    pub callee: String,
    pub file: String,
    pub line: u32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RouteRefFacts {
    pub route_refs: Vec<RouteRef>,
    pub route_helpers: Vec<RouteHelper>,
    pub route_helper_imports: Vec<RouteHelperImport>,
    pub route_helper_refs: Vec<RouteHelperRef>,
}

/// Scan `source` for route references. Returns a Vec of RouteRef.
pub fn extract_route_refs(source: &str, file: &str) -> Vec<RouteRef> {
    extract_route_ref_facts(source, file).route_refs
}

pub fn extract_route_ref_facts(source: &str, file: &str) -> RouteRefFacts {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(Path::new(file)).unwrap_or(SourceType::tsx());
    let ret = Parser::new(&allocator, source, source_type).parse();

    extract_route_ref_facts_from_program(&ret.program, source, file)
}

pub fn extract_route_refs_from_program<'a>(
    program: &Program<'a>,
    source: &str,
    file: &str,
) -> Vec<RouteRef> {
    extract_route_ref_facts_from_program(program, source, file).route_refs
}

pub fn extract_route_ref_facts_from_program<'a>(
    program: &Program<'a>,
    source: &str,
    file: &str,
) -> RouteRefFacts {
    let mut router_bindings = collect_import_bindings(&program.body);
    collect_router_bindings_for_scope(&program.body, &mut router_bindings);

    let mut refs = Vec::new();
    for stmt in &program.body {
        collect_from_statement(stmt, source, file, &mut router_bindings, &mut refs);
    }

    let route_helpers = collect_route_helpers(program);
    let route_helper_imports = collect_route_helper_imports(program);
    let route_helper_refs = collect_route_helper_refs_from_program(program, source, file);

    RouteRefFacts {
        route_refs: refs,
        route_helpers,
        route_helper_imports,
        route_helper_refs,
    }
}

#[derive(Clone, Default)]
struct RouterBindings<'a> {
    objects: HashSet<&'a str>,
    methods: HashSet<&'a str>,
    redirects: HashSet<&'a str>,
}
