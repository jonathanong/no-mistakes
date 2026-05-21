use super::ServerRouteVisitor;
use crate::server_routes::types::Framework;
use oxc_allocator::Allocator;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::path::Path;

pub(crate) fn has_server_route_shape(path: &Path, source: &str) -> bool {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap_or_else(|_| SourceType::ts());
    let parsed = Parser::new(&allocator, source, source_type).parse();
    if parsed.panicked || !parsed.errors.is_empty() {
        return false;
    }

    let mut visitor = ServerRouteVisitor::new(path, source);
    visitor.visit_program(&parsed.program);
    let facts = visitor.facts;
    let has_known_route = facts.routes.iter().any(|route| {
        facts
            .bindings
            .get(&route.binding)
            .is_some_and(|binding| binding.framework != Framework::Heuristic)
    });
    let has_known_mount = facts.mounts.iter().any(|mount| {
        let parent_known = facts
            .bindings
            .get(&mount.parent)
            .is_some_and(|binding| binding.framework != Framework::Heuristic);
        let child_known = facts
            .bindings
            .get(&mount.child)
            .is_some_and(|binding| binding.framework != Framework::Heuristic);
        parent_known && child_known
    });
    has_known_route || has_known_mount
}
