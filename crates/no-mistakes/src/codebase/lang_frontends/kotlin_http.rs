use regex::Regex;
use std::sync::OnceLock;

pub(super) fn extract_http_routes(source: &str) -> Vec<(String, String)> {
    let prefix = super::super::http_routes::class_request_mapping(source, kotlin_class_re());
    let mut routes =
        super::super::http_routes::extract_method_mappings(source, method_mapping_re());
    for (path, _) in &mut routes {
        *path = super::super::http_routes::join_route(prefix.as_deref(), path);
    }
    routes.sort();
    routes.dedup();
    routes
}

fn kotlin_class_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b(?:class|interface|object)\s+[A-Za-z_]").expect("class"))
}

fn method_mapping_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?s)@(?:(?:Get|Post|Put|Patch|Delete)Mapping|RequestMapping)\(\s*(?:(?:path|value)\s*=\s*)?"([^"]+)"\s*\)(?:\s*@[A-Za-z_.]+(?:\([^)]*\))?)*\s*(?:(?:public|private|protected|internal|open|override|suspend|inline|tailrec|actual|expect)\s+)*fun\s+([A-Za-z_][A-Za-z0-9_]*)\s*\("#,
        )
        .expect("method mapping")
    })
}

#[cfg(test)]
#[path = "kotlin_http_tests.rs"]
mod tests;
