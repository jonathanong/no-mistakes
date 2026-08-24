mod parser;
mod read;
use super::scanner::{
    find_label_colon, find_matching_delimiter, read_quoted_string, string_arg, Scanner,
};
use parser::target_calls;
pub(super) use read::{
    parse_local_package_bindings, parse_local_package_paths, parse_manifest_products,
    parse_manifest_targets,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SwiftManifestDiagnostic {
    UnsupportedDynamicDeclaration,
}

/// Classify a `Package.swift` delta without treating formatting as a dependency
/// change. Only static package declarations and static target/product/plugin
/// bindings are removed from the compared projection; every other manifest field
/// remains structural configuration and therefore stays broad.
pub(crate) fn dependency_only_manifest_change(
    before: &str,
    after: &str,
) -> Result<bool, SwiftManifestDiagnostic> {
    Ok(manifest_projection(before)? == manifest_projection(after)?
        && normalize_manifest(before) != normalize_manifest(after))
}

pub(crate) fn formatting_only_manifest_change(before: &str, after: &str) -> bool {
    normalize_manifest(before) == normalize_manifest(after)
}

fn normalize_manifest(source: &str) -> String {
    let mut normalized = String::new();
    let mut scanner = Scanner::new(source);
    while let Some(index) = scanner.next_code_index() {
        let ch = source[index..]
            .chars()
            .next()
            .expect("scanner indices are character boundaries");
        if ch == '"' {
            if let Some((_, end)) = read_quoted_string(source, index) {
                normalized.push_str(&source[index..end]);
            } else {
                normalized.push_str(&source[index..]);
                break;
            }
        } else if !ch.is_whitespace() {
            normalized.push(ch);
        }
    }
    normalized
}

fn manifest_projection(source: &str) -> Result<String, SwiftManifestDiagnostic> {
    let mut ignored = package_declaration_ranges(source)?;
    for call in target_calls(source) {
        for label in ["dependencies", "plugins"] {
            if let Some((start, end, body)) = labeled_array_range(call.body, label) {
                validate_binding_list(body)?;
                let body_offset = call.body.as_ptr() as usize - source.as_ptr() as usize;
                ignored.push((body_offset + start, body_offset + end));
            }
        }
    }
    ignored.sort_unstable();
    let mut projection = String::new();
    let mut cursor = 0;
    for (start, end) in ignored {
        if start < cursor {
            continue;
        }
        projection.push_str(&normalize_manifest(&source[cursor..start]));
        cursor = end;
    }
    projection.push_str(&normalize_manifest(&source[cursor..]));
    Ok(projection)
}

fn package_declaration_ranges(
    source: &str,
) -> Result<Vec<(usize, usize)>, SwiftManifestDiagnostic> {
    let mut ranges = Vec::new();
    let mut scanner = Scanner::new(source);
    while let Some(index) = scanner.next_code_index() {
        if !source[index..].starts_with(".package(") {
            continue;
        }
        let open = index + ".package".len();
        let close = find_matching_delimiter(source, open, '(', ')')
            .ok_or(SwiftManifestDiagnostic::UnsupportedDynamicDeclaration)?;
        validate_package_declaration(&source[open + 1..close])?;
        ranges.push((index, close + 1));
        scanner.skip_to(close + 1);
    }
    Ok(ranges)
}

fn validate_package_declaration(body: &str) -> Result<(), SwiftManifestDiagnostic> {
    let path = string_arg(body, "path");
    let url = string_arg(body, "url");
    if path.is_none() && url.is_none() {
        return Err(SwiftManifestDiagnostic::UnsupportedDynamicDeclaration);
    }
    for label in ["path", "url", "from", "exact", "branch", "revision"] {
        if find_label_colon(body, label).is_some() && string_arg(body, label).is_none() {
            return Err(SwiftManifestDiagnostic::UnsupportedDynamicDeclaration);
        }
    }
    if body.contains("requirement:") || body.contains("condition:") || body.contains("platforms:") {
        return Err(SwiftManifestDiagnostic::UnsupportedDynamicDeclaration);
    }
    Ok(())
}

fn labeled_array_range<'a>(body: &'a str, label: &str) -> Option<(usize, usize, &'a str)> {
    let colon = find_label_colon(body, label)?;
    let start = body[colon + 1..]
        .char_indices()
        .find_map(|(offset, ch)| (!ch.is_whitespace()).then_some(colon + 1 + offset))?;
    (body[start..].starts_with('[')).then_some(())?;
    let end = find_matching_delimiter(body, start, '[', ']')?;
    Some((start, end + 1, &body[start + 1..end]))
}

fn validate_binding_list(body: &str) -> Result<(), SwiftManifestDiagnostic> {
    for binding in split_top_level_bindings(body) {
        validate_binding(binding.trim())?;
    }
    Ok(())
}

fn split_top_level_bindings(body: &str) -> Vec<&str> {
    let mut bindings = Vec::new();
    let mut start = 0;
    let mut depth = 0usize;
    let mut scanner = Scanner::new(body);
    while let Some(index) = scanner.next_code_index() {
        let ch = body[index..]
            .chars()
            .next()
            .expect("scanner index is valid");
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                bindings.push(&body[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    if !body[start..].trim().is_empty() {
        bindings.push(&body[start..]);
    }
    bindings
}

fn validate_binding(binding: &str) -> Result<(), SwiftManifestDiagnostic> {
    if binding.starts_with('"') {
        let Some((_, end)) = read_quoted_string(binding, 0) else {
            return Err(SwiftManifestDiagnostic::UnsupportedDynamicDeclaration);
        };
        return (binding[end..].trim().is_empty())
            .then_some(())
            .ok_or(SwiftManifestDiagnostic::UnsupportedDynamicDeclaration);
    }
    for call in [".target", ".product", ".byName", ".plugin"] {
        if !binding.starts_with(call) {
            continue;
        }
        let open = call.len();
        let Some(close) = find_matching_delimiter(binding, open, '(', ')') else {
            return Err(SwiftManifestDiagnostic::UnsupportedDynamicDeclaration);
        };
        let args = &binding[open + 1..close];
        if !binding[close + 1..].trim().is_empty() || string_arg(args, "name").is_none() {
            return Err(SwiftManifestDiagnostic::UnsupportedDynamicDeclaration);
        }
        if call == ".product" && string_arg(args, "package").is_none() {
            return Err(SwiftManifestDiagnostic::UnsupportedDynamicDeclaration);
        }
        return Ok(());
    }
    Err(SwiftManifestDiagnostic::UnsupportedDynamicDeclaration)
}

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
