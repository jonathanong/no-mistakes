pub(super) fn route_from_routes_rel(rel_under_routes: &str) -> Option<String> {
    let rel = rel_under_routes.replace('\\', "/");
    let stem = strip_ts_js_extension(&rel)?;
    if stem.ends_with(".server") || stem.ends_with(".client") {
        return None;
    }
    Some(url_from_flat(&stem.replace('/', ".")))
}

pub(super) fn route_from_app_root(rel: &str) -> Option<String> {
    let rel = rel.replace('\\', "/");
    let stem = strip_ts_js_extension(&rel)?;
    (stem == "app/root").then(|| "/".to_string())
}

fn strip_ts_js_extension(path: &str) -> Option<&str> {
    for extension in [".tsx", ".ts", ".jsx", ".js", ".mts", ".cts", ".mjs", ".cjs"] {
        if let Some(stem) = path.strip_suffix(extension) {
            return Some(stem);
        }
    }
    None
}

fn url_from_flat(id: &str) -> String {
    let mut segments = Vec::new();
    for part in id.split('.') {
        if part.is_empty() || part == "_index" || part == "index" {
            continue;
        }
        if part.starts_with('_') {
            continue;
        }
        let part = part.strip_suffix('_').unwrap_or(part);
        if part == "$" {
            segments.push("*".to_string());
        } else if let Some(name) = part.strip_prefix('$') {
            segments.push(format!(":{name}"));
        } else {
            segments.push(part.to_string());
        }
    }
    if segments.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", segments.join("/"))
    }
}
