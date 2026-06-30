fn resolved_backend_pattern(options: &GraphConfigOptions) -> Option<String> {
    if !options.http_route.backend_pattern.is_empty() {
        Some(options.http_route.backend_pattern.clone())
    } else {
        route_backend_pattern(options)
    }
}

fn resolved_backend_register_object(options: &GraphConfigOptions) -> Option<String> {
    if !options.http_route.register_object.is_empty() {
        Some(options.http_route.register_object.clone())
    } else {
        route_backend_register_object(options)
    }
}

fn resolved_backend_prefixes(options: &GraphConfigOptions) -> Vec<String> {
    if !options.http_call.backend_prefixes.is_empty() {
        options.http_call.backend_prefixes.clone()
    } else {
        route_backend_prefixes(options)
    }
}

fn route_backend_prefixes(options: &GraphConfigOptions) -> Vec<String> {
    options.route.backend_prefixes.clone()
}

fn route_backend_pattern(options: &GraphConfigOptions) -> Option<String> {
    (!options.route.backend_pattern.is_empty()).then(|| options.route.backend_pattern.clone())
}

fn route_backend_register_object(options: &GraphConfigOptions) -> Option<String> {
    (!options.route.backend_register_object.is_empty())
        .then(|| options.route.backend_register_object.clone())
}
