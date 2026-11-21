mod client;

use std::path::Path;

pub(super) use client::client_call_lines_from_program;

pub(super) fn has_server_like_route_call_from_program(
    path: &Path,
    source: &str,
    program: &oxc_ast::ast::Program<'_>,
) -> bool {
    crate::server_routes::has_server_route_shape_from_program(path, source, program)
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
pub(super) mod test_support;
