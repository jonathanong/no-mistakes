use super::*;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::relative_slash_path;
use globset::Glob;

pub(super) fn reference_exists(
    root: &Path,
    config_file: &Path,
    opts: &Options,
    reference: &str,
    rel_files: &[String],
) -> Result<bool> {
    let base = if opts.base_dir == BaseDir::Root {
        root.to_path_buf()
    } else {
        config_file.parent().unwrap_or(root).to_path_buf()
    };
    let target = normalize_path(&base.join(reference));
    if target.starts_with(root) && tracked_reference_exists(root, &target, rel_files) {
        return Ok(true);
    }
    if opts.allow_globs && has_glob_metachar(reference) {
        let pattern = reference_pattern(root, config_file, opts, reference);
        let glob = Glob::new(&pattern)?;
        let matcher = glob.compile_matcher();
        return Ok(rel_files.iter().any(|rel| matcher.is_match(rel)));
    }
    Ok(false)
}

fn tracked_reference_exists(root: &Path, target: &Path, rel_files: &[String]) -> bool {
    let rel = relative_slash_path(root, target);
    let rel = rel.trim_start_matches("./").trim_end_matches('/');
    if rel.is_empty() || rel == "." {
        return !rel_files.is_empty();
    }
    rel_files
        .iter()
        .any(|candidate| candidate == rel || candidate.starts_with(&format!("{rel}/")))
}

pub(super) fn has_glob_metachar(reference: &str) -> bool {
    let mut escaped = false;
    for ch in reference.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if matches!(ch, '*' | '?' | '{') {
            return true;
        }
    }
    false
}

pub(super) fn reference_pattern(
    root: &Path,
    config_file: &Path,
    opts: &Options,
    reference: &str,
) -> String {
    if opts.base_dir == BaseDir::Root {
        return normalize_glob_pattern(reference);
    }
    let Some(parent) = config_file.parent() else {
        return normalize_glob_pattern(reference);
    };
    let dir = relative_slash_path(root, parent);
    if dir.is_empty() {
        normalize_glob_pattern(reference)
    } else {
        normalize_glob_pattern(&format!("{}/{reference}", glob_escape_literal(&dir)))
    }
}

fn normalize_glob_pattern(pattern: &str) -> String {
    let mut parts = Vec::new();
    for part in pattern.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if !parts.is_empty() && parts.last() != Some(&"..") {
                    parts.pop();
                } else {
                    parts.push(part);
                }
            }
            _ => parts.push(part),
        }
    }
    parts.join("/")
}

fn glob_escape_literal(value: &str) -> String {
    value
        .chars()
        .flat_map(|ch| {
            if matches!(ch, '*' | '?' | '[' | ']' | '{' | '}' | '\\') {
                vec!['\\', ch]
            } else {
                vec![ch]
            }
        })
        .collect()
}
