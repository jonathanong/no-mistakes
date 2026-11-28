use globset::{GlobBuilder, GlobMatcher};
use serde_yaml::Value;

pub(super) struct CompiledGlob {
    negate: bool,
    matcher: GlobMatcher,
}

pub(super) fn compile_pattern_sequence(raw: &Value) -> Result<Vec<CompiledGlob>, String> {
    let Some(items) = raw.as_sequence() else {
        return Err("is not a sequence of path filter patterns.".to_string());
    };
    let mut globs = Vec::with_capacity(items.len());
    for item in items {
        let Some(pattern) = item.as_str() else {
            return Err("contains a non-string pattern.".to_string());
        };
        if pattern.contains("${{") {
            return Err(format!("contains dynamic expression `{pattern}`."));
        }
        if has_extglob(pattern) {
            return Err(format!("uses unsupported extglob syntax `{pattern}`."));
        }
        if has_github_quantifier(pattern) {
            return Err(format!(
                "uses GitHub `?` or `+` quantifiers in `{pattern}`, which this check does not evaluate."
            ));
        }
        if has_embedded_globstar(pattern) {
            return Err(format!(
                "uses embedded `**` in `{pattern}`, which this check does not evaluate."
            ));
        }
        let (negate, body) = match pattern.strip_prefix('!') {
            Some(rest) => (true, rest),
            None => (false, pattern),
        };
        let body = body.strip_prefix("./").unwrap_or(body);
        if body.is_empty() {
            return Err(format!("contains empty pattern `{pattern}`."));
        }
        let normalized = normalize_github_glob(body);
        let glob = GlobBuilder::new(&normalized)
            .literal_separator(true)
            .build()
            .map_err(|err| format!("contains invalid glob `{pattern}` ({err})."))?;
        globs.push(CompiledGlob {
            negate,
            matcher: glob.compile_matcher(),
        });
    }
    Ok(globs)
}

pub(super) fn selected_by(globs: &[CompiledGlob], path: &str) -> bool {
    let mut selected = false;
    for glob in globs {
        if glob.matcher.is_match(path) {
            selected = !glob.negate;
        }
    }
    selected
}

fn has_extglob(pattern: &str) -> bool {
    ["+(", "?(", "!(", "@(", "*("]
        .iter()
        .any(|marker| pattern.contains(marker))
}

fn has_github_quantifier(pattern: &str) -> bool {
    let mut class = false;
    for ch in pattern.chars() {
        match ch {
            '[' if !class => class = true,
            ']' if class => class = false,
            '?' | '+' if !class => return true,
            _ => {}
        }
    }
    false
}

fn has_embedded_globstar(pattern: &str) -> bool {
    let bytes = pattern.as_bytes();
    let mut index = 0;
    while index + 1 < bytes.len() {
        if bytes[index] == b'*' && bytes[index + 1] == b'*' {
            let before_boundary = index == 0 || bytes[index - 1] == b'/';
            let after = index + 2;
            let after_boundary = after >= bytes.len() || bytes[after] == b'/';
            if !before_boundary && !after_boundary {
                return true;
            }
            index += 2;
            continue;
        }
        index += 1;
    }
    false
}

fn normalize_github_glob(pattern: &str) -> String {
    let chars: Vec<char> = pattern.chars().collect();
    let mut out = String::with_capacity(pattern.len() + 2);
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '*' && chars.get(i + 1) == Some(&'*') {
            out.push_str("**");
            i += 2;
            if matches!(chars.get(i), Some(c) if *c != '/') {
                out.push_str("/*");
            }
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

pub(super) fn find_top_key_line(source: &str, key: &str) -> Option<usize> {
    source.lines().enumerate().find_map(|(idx, line)| {
        if line.starts_with([' ', '\t', '#']) {
            return None;
        }
        let (raw_key, _) = line.split_once(':')?;
        let clean = raw_key.trim().trim_matches(['\'', '"']);
        (clean == key).then_some(idx + 1)
    })
}

pub(super) fn find_key_line_after(source: &str, start_line: usize, key: &str) -> Option<usize> {
    source
        .lines()
        .enumerate()
        .skip(start_line.saturating_sub(1))
        .find_map(|(idx, line)| {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                return None;
            }
            let (raw_key, _) = trimmed.split_once(':')?;
            let clean = raw_key.trim().trim_matches(['\'', '"']);
            (clean == key).then_some(idx + 1)
        })
}
