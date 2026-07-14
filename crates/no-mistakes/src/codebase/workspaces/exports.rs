fn exports_to_entry_path(exports: &serde_json::Value) -> Option<String> {
    match exports {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Object(map) => {
            if let Some(dot) = map.get(".") {
                return exports_to_entry_path(dot);
            }
            ["import", "default", "require", "types"]
                .iter()
                .find_map(|key| map.get(*key).and_then(exports_to_entry_path))
        }
        _ => None,
    }
}

#[inline(never)]
fn resolve_export_subpath(exports: &serde_json::Value, subpath: &str) -> Option<String> {
    let serde_json::Value::Object(map) = exports else {
        return None;
    };

    if let Some(value) = map.get(subpath) {
        return exports_to_entry_path(value);
    }

    let mut patterns = Vec::new();
    for (pattern, value) in map {
        if let Some(star_idx) = pattern.find('*') {
            patterns.push((pattern, value, star_idx));
        }
    }
    patterns.sort_by(compare_export_patterns);

    for (pattern, value, star_idx) in patterns {
        if pattern[star_idx + 1..].contains('*') {
            continue;
        }
        let prefix = &pattern[..star_idx];
        let suffix = &pattern[star_idx + 1..];
        let Some(capture) = subpath
            .strip_prefix(prefix)
            .and_then(|rest| rest.strip_suffix(suffix))
        else {
            continue;
        };
        let Some(target) = exports_to_entry_path(value) else {
            continue;
        };
        if target.matches('*').count() == 1 {
            return Some(target.replacen('*', capture, 1));
        }
    }

    None
}

fn compare_export_patterns(
    (a, _, a_star): &(&String, &serde_json::Value, usize),
    (b, _, b_star): &(&String, &serde_json::Value, usize),
) -> Ordering {
    let star_order = b_star.cmp(a_star);
    if star_order != Ordering::Equal {
        return star_order;
    }
    a.cmp(b)
}

#[inline(never)]
fn package_name_and_subpath(specifier: &str) -> Option<(String, Option<String>)> {
    if specifier.starts_with('.') || specifier.starts_with('/') {
        return None;
    }

    let mut parts = specifier.splitn(3, '/');
    let first = parts.next().unwrap_or("");
    if first.starts_with('@') {
        let scope_pkg = parts.next()?;
        let name_len = first.len() + 1 + scope_pkg.len();
        let subpath = specifier
            .get(name_len + 1..)
            .map(|rest| format!("./{rest}"));
        return Some((specifier[..name_len].to_string(), subpath));
    }

    let subpath = specifier
        .get(first.len() + 1..)
        .map(|rest| format!("./{rest}"));
    Some((first.to_string(), subpath))
}
