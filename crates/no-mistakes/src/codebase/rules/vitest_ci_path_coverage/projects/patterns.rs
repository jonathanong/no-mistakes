pub(super) fn project_root_patterns(project_root: &str) -> Vec<String> {
    let root = normalize_project_glob_part(project_root);
    if root.is_empty() || root == "." {
        vec!["**".to_string()]
    } else {
        vec![format!("{root}/**")]
    }
}

pub(super) fn project_relative_pattern(project_root: &str, pattern: &str) -> String {
    let (negated, pattern) = pattern
        .trim()
        .strip_prefix('!')
        .map_or((false, pattern.trim()), |pattern| (true, pattern));
    let root = normalize_project_glob_part(project_root);
    let pattern = normalize_project_glob_part(pattern);
    let joined = if root.is_empty() || root == "." || pattern.starts_with(&format!("{root}/")) {
        pattern
    } else {
        format!("{root}/{pattern}")
    };
    if negated {
        format!("!{joined}")
    } else {
        joined
    }
}

pub(super) fn normalize_project_glob_part(raw: &str) -> String {
    let mut part = raw.trim().trim_matches('/').to_string();
    while let Some(rest) = part.strip_prefix("./") {
        part = rest.to_string();
    }
    part
}
