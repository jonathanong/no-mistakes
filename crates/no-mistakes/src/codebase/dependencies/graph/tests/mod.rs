fn package_name_from_spec(spec: &str) -> &str {
    if spec.starts_with('@') {
        let after_scope = spec.trim_start_matches('@');
        let slash_idx = after_scope.find('/').map(|i| i + 1);
        if let Some(idx) = slash_idx {
            let after_first_slash = &after_scope[idx..];
            let end = after_first_slash
                .find('/')
                .map(|i| idx + i + 1)
                .unwrap_or(spec.len());
            &spec[..end]
        } else {
            spec
        }
    } else {
        match spec.find('/') {
            Some(idx) => &spec[..idx],
            None => spec,
        }
    }
}

include!("core.rs");
include!("route_import.rs");
include!("extra_cases.rs");
include!("extra_playwright_routes.rs");
include!("extra_selector.rs");
include!("extra_symbol_scoped.rs");
include!("extra_symbol_defensive.rs");
include!("extra_symbol_helpers.rs");
include!("extra_symbol.rs");
include!("types.rs");

mod selector_fact_plan;
mod selector_optimization;
