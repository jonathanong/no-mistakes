use super::super::scanner::{find_label_colon, find_matching_delimiter, string_arg, Scanner};
use super::super::SwiftTargetFacts;
use super::parser::{dependencies_body, manifest_dependencies};

pub(crate) fn parse_manifest_targets(source: &str) -> Vec<SwiftTargetFacts> {
    super::parser::target_calls(source)
        .into_iter()
        .filter_map(|call| {
            let name = string_arg(call.body, "name")?;
            let (dependencies, product_packages) = dependencies_body(call.body)
                .map(manifest_dependencies)
                .unwrap_or_default();
            Some(SwiftTargetFacts {
                name,
                is_test: call.is_test,
                dependencies,
                product_packages,
                roots: Vec::new(),
            })
        })
        .collect()
}

pub(crate) fn parse_manifest_products(
    source: &str,
) -> std::collections::BTreeMap<String, Vec<String>> {
    let mut products = std::collections::BTreeMap::new();
    let mut scanner = Scanner::new(source);
    while let Some(index) = scanner.next_code_index() {
        let rest = &source[index..];
        let Some(call) = [".library(", ".executable(", ".plugin("]
            .into_iter()
            .find(|call| rest.starts_with(call))
        else {
            continue;
        };
        let open = index + call.len() - 1;
        let Some(close) = find_matching_delimiter(source, open, '(', ')') else {
            continue;
        };
        let body = &source[open + 1..close];
        if let (Some(name), Some(targets)) = (
            string_arg(body, "name"),
            labeled_array_body(body, "targets"),
        ) {
            products.insert(name, manifest_dependencies(targets).0);
        }
        scanner.skip_to(close + 1);
    }
    products
}

fn labeled_array_body<'a>(body: &'a str, label: &str) -> Option<&'a str> {
    let colon = find_label_colon(body, label)?;
    let open = body[colon + 1..]
        .char_indices()
        .find_map(|(offset, ch)| (!ch.is_whitespace()).then_some((colon + 1 + offset, ch)))?;
    (open.1 == '[').then_some(())?;
    let close = find_matching_delimiter(body, open.0, '[', ']')?;
    body.get(open.0 + 1..close)
}

pub(crate) fn parse_local_package_paths(source: &str) -> Vec<String> {
    parse_local_package_bindings(source).into_keys().collect()
}

pub(crate) fn parse_local_package_bindings(
    source: &str,
) -> std::collections::BTreeMap<String, String> {
    let mut paths = std::collections::BTreeMap::new();
    let mut scanner = Scanner::new(source);
    while let Some(index) = scanner.next_code_index() {
        if !source[index..].starts_with(".package(") {
            continue;
        }
        let open = index + ".package".len();
        if let Some(close) = find_matching_delimiter(source, open, '(', ')') {
            let body = &source[open + 1..close];
            if let Some(path) = string_arg(body, "path") {
                let identity = string_arg(body, "name")
                    .unwrap_or_else(|| {
                        path.trim_end_matches('/')
                            .rsplit('/')
                            .next()
                            .unwrap_or(&path)
                            .to_string()
                    })
                    .to_ascii_lowercase();
                paths.insert(path, identity);
            }
            scanner.skip_to(close + 1);
        }
    }
    paths
}
