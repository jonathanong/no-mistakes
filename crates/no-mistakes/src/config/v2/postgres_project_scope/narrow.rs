use anyhow::{bail, Result};
use globset::GlobBuilder;

/// Project-root prefix used when a repository-relative pattern is required.
pub(super) fn root_prefix(root: Option<&str>) -> Option<&str> {
    let root = root?.trim().trim_matches('/');
    if root.is_empty() || root == "." {
        None
    } else {
        Some(root)
    }
}

pub(super) fn prefix_pattern(root: Option<&str>, pattern: &str) -> String {
    let pattern = pattern.trim().trim_start_matches("./");
    match root_prefix(root) {
        Some(root) if !pattern.is_empty() => format!("{root}/{pattern}"),
        Some(root) => root.to_string(),
        None => pattern.to_string(),
    }
}

/// `rule` is proven narrower than `project` when it is equal, or when
/// `project` ends in `/**` and `rule` stays under that directory with no `..`.
pub(super) fn proven_narrower(rule: &str, project: &str) -> bool {
    if rule == project {
        return true;
    }
    let Some(directory) = project.strip_suffix("/**") else {
        return false;
    };
    if has_parent_segment(rule) {
        return false;
    }
    if directory.is_empty() {
        return !rule.starts_with('/');
    }
    rule.starts_with(&format!("{directory}/"))
}

pub(super) fn escapes_project_root(root: Option<&str>, pattern: &str) -> bool {
    let form = prefix_pattern(root, pattern);
    let mut stack = Vec::new();
    for part in form.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            if stack.pop().is_none() {
                return true;
            }
            continue;
        }
        stack.push(part);
    }
    match root_prefix(root) {
        Some(root) => {
            let root_parts: Vec<&str> = root.split('/').filter(|part| !part.is_empty()).collect();
            stack.len() < root_parts.len() || stack[..root_parts.len()] != root_parts[..]
        }
        None => false,
    }
}

pub(super) fn prove_patterns(
    rule: &str,
    label: &str,
    patterns: &[String],
    allowed: &[String],
) -> Result<()> {
    for pattern in patterns {
        validate_glob(pattern, &format!("{rule} {label}"))?;
        if !allowed
            .iter()
            .any(|project| proven_narrower(pattern, project))
        {
            bail!(
                "{rule} {label} `{pattern}` is not a proven narrower glob; an unproven pattern is a config error"
            );
        }
    }
    Ok(())
}

pub(super) fn validate_glob(pattern: &str, key: &str) -> Result<()> {
    GlobBuilder::new(pattern.trim_start_matches("./"))
        .literal_separator(false)
        .build()
        .map(|_| ())
        .map_err(|err| anyhow::anyhow!("{key} contains invalid glob `{pattern}`: {err}"))
}

fn has_parent_segment(pattern: &str) -> bool {
    pattern.split('/').any(|part| part == "..")
}
