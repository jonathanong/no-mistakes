use oxc_allocator::Allocator;
use oxc_span::SourceType;
use std::path::Path;

pub(in crate::codebase::rules::server_route_client_boundary) fn has_server_like_route_call(
    path: &Path,
    source: &str,
) -> bool {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap_or_else(|_| SourceType::ts());
    let parsed = crate::ast::parse(path, &allocator, source, source_type);
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return false;
    }
    super::has_server_like_route_call_from_program(path, source, &parsed.program)
}

pub(in crate::codebase::rules::server_route_client_boundary) fn client_call_lines(
    path: &Path,
    source: &str,
) -> Vec<usize> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap_or_else(|_| SourceType::ts());
    let parsed = crate::ast::parse(path, &allocator, source, source_type);
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return Vec::new();
    }
    super::client_call_lines_from_program(source, &parsed.program)
}
