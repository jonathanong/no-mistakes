pub(super) fn rust_path_prefixes(import: &str) -> Vec<String> {
    let parts: Vec<&str> = import.split('.').filter(|part| !part.is_empty()).collect();
    (1..parts.len()).map(|end| parts[..end].join(".")).collect()
}

pub(super) fn qualify_rust_use(kind: &str, item: &str, module: Option<&str>) -> String {
    match kind {
        "self" => match module {
            Some(module) => format!("{module}.{item}"),
            None => item.to_string(),
        },
        "super" => match module.and_then(|module| module.rsplit_once('.')) {
            Some((parent, _)) => format!("{parent}.{item}"),
            None => item.to_string(),
        },
        _ => item.to_string(),
    }
}

pub(super) fn expand_rust_use(tree: &str) -> Vec<String> {
    let tree = tree.trim();
    let Some(start) = tree.find('{') else {
        return vec![tree.to_string()];
    };
    let Some(end) = tree.rfind('}') else {
        return vec![tree.to_string()];
    };
    let prefix = tree[..start].trim_end_matches(':');
    split_use_members(&tree[start + 1..end])
        .into_iter()
        .flat_map(|member| {
            let member = member
                .split_whitespace()
                .take_while(|token| !token.eq_ignore_ascii_case("as"))
                .collect::<Vec<_>>()
                .join(" ");
            let member = member.trim();
            if member.is_empty() || member == "self" {
                return if prefix.is_empty() {
                    Vec::new()
                } else {
                    vec![prefix.to_string()]
                };
            }
            let combined = if prefix.is_empty() {
                member.to_string()
            } else {
                format!("{prefix}::{member}")
            };
            expand_rust_use(&combined)
        })
        .collect()
}

fn split_use_members(inner: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    for ch in inner.chars() {
        match ch {
            '{' => {
                depth += 1;
                current.push(ch);
            }
            '}' => {
                depth -= 1;
                current.push(ch);
            }
            ',' if depth == 0 => {
                if !current.trim().is_empty() {
                    parts.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }
    if !current.trim().is_empty() {
        parts.push(current);
    }
    parts
}
