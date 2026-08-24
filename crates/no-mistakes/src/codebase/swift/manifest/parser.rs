use super::super::scanner::{
    find_label_colon, find_matching_delimiter, read_quoted_string, string_arg, Scanner,
};

pub(super) struct TargetCall<'a> {
    pub(super) is_test: bool,
    pub(super) default_source_root: &'static str,
    pub(super) body: &'a str,
}

pub(super) fn target_calls(source: &str) -> Vec<TargetCall<'_>> {
    let mut calls = Vec::new();
    let mut scanner = Scanner::new(source);
    while let Some(index) = scanner.next_code_index() {
        let rest = &source[index..];
        let (is_test, default_source_root, requires_capability, open_index) =
            if rest.starts_with(".testTarget(") {
                (true, "Tests", false, index + ".testTarget".len())
            } else if rest.starts_with(".executableTarget(") {
                (false, "Sources", false, index + ".executableTarget".len())
            } else if rest.starts_with(".macro(") {
                (false, "Sources", false, index + ".macro".len())
            } else if rest.starts_with(".plugin(") {
                (false, "Plugins", true, index + ".plugin".len())
            } else if rest.starts_with(".target(") {
                (false, "Sources", false, index + ".target".len())
            } else {
                continue;
            };
        if let Some(close_index) = find_matching_delimiter(source, open_index, '(', ')') {
            let body = &source[open_index + 1..close_index];
            if requires_capability && find_label_colon(body, "capability").is_none() {
                scanner.skip_to(close_index + 1);
                continue;
            }
            calls.push(TargetCall {
                is_test,
                default_source_root,
                body,
            });
            scanner.skip_to(close_index + 1);
        }
    }
    calls
}

pub(super) fn dependencies_body(target_body: &str) -> Option<&str> {
    let label = find_label_colon(target_body, "dependencies")?;
    let open_bracket = target_body[label + 1..]
        .char_indices()
        .find_map(|(offset, ch)| (!ch.is_whitespace()).then_some((label + 1 + offset, ch)))?;
    if open_bracket.1 != '[' {
        return None;
    }
    let close_bracket = find_matching_delimiter(target_body, open_bracket.0, '[', ']')?;
    target_body.get(open_bracket.0 + 1..close_bracket)
}

pub(super) fn manifest_dependencies(
    dependencies_body: &str,
) -> (Vec<String>, std::collections::BTreeMap<String, String>) {
    let mut names = Vec::new();
    let mut product_packages = std::collections::BTreeMap::new();
    let mut scanner = Scanner::new(dependencies_body);
    while let Some(index) = scanner.next_code_index() {
        let rest = &dependencies_body[index..];
        if rest.starts_with('"') {
            if let Some((value, next)) = read_quoted_string(dependencies_body, index) {
                names.push(value);
                scanner.skip_to(next);
            }
            continue;
        }
        let open = [".target", ".product", ".byName"]
            .into_iter()
            .find_map(|name| {
                rest.starts_with(&format!("{name}("))
                    .then_some(index + name.len())
            });
        if let Some(open) = open {
            if let Some(close) = find_matching_delimiter(dependencies_body, open, '(', ')') {
                if let Some(name) = string_arg(&dependencies_body[open + 1..close], "name") {
                    if rest.starts_with(".product(") {
                        if let Some(package) =
                            string_arg(&dependencies_body[open + 1..close], "package")
                        {
                            product_packages.insert(name.clone(), package.to_ascii_lowercase());
                        }
                    }
                    names.push(name);
                }
                scanner.skip_to(close + 1);
            }
        }
    }
    (names, product_packages)
}
