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
