mod client;

use std::path::Path;

pub(super) use client::client_call_lines;

pub(super) fn has_server_like_route_call(path: &Path, source: &str) -> bool {
    crate::server_routes::has_server_route_shape(path, source)
}
