use super::ServerRouteVisitor;
use crate::server_routes::types::Framework;
use oxc_ast::ast::Program;
use oxc_ast_visit::Visit;
use std::path::Path;

pub(crate) fn has_server_route_shape_from_program(
    path: &Path,
    source: &str,
    program: &Program<'_>,
) -> bool {
    let mut visitor = ServerRouteVisitor::new(path, source);
    visitor.visit_program(program);
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
